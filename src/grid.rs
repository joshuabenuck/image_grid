use failure::Error;
use glutin_window::GlutinWindow as Window;
use graphics::{DrawState, Image, ImageSize, Transformed};
use opengl_graphics::{GlGraphics, Texture};
use piston::event_loop::*;
use piston::input::{
    keyboard::{Key, ModifierKey},
    mouse::MouseButton,
    Button, MouseCursorEvent, MouseScrollEvent, PressEvent, ReleaseEvent, RenderArgs, RenderEvent,
    UpdateEvent,
};
use piston::window::AdvancedWindow;
use std::cmp::{max, min};
use std::process::Child;

pub type Color = [f32; 4];

pub enum TileAction {
    None,
    Launch(Result<Child, Error>),
}

pub struct TileState {
    width: u16,
    height: u16,
    scale: f32,
}

impl TileState {
    fn update_size(&mut self, image: &Texture, w_h: f32, h_h: f32) -> (f32, f32) {
        (0.0, 0.0)
    }
}

pub trait TileHandler {
    fn window_title(&self) -> String;

    fn tile_size(&self) -> (u16, u16) {
        self.compute_tile_size()
    };

    fn tile_size_hint(&self) -> (u16, u16);

    fn compute_size(&self, i: usize, w: f32, h: f32) -> (f32, f32, f32) {
        let (width, height) = self.tile(i).get_size();
        let scale = f32::min(w / width as f32, h / height as f32);
        let width = width as f32 * scale;
        let height = height as f32 * scale;
        (scale, width, height)
    }

    fn compute_tile_size(&self) -> (u16, u16) {
        let mut current_width = 0;
        let mut current_height = 0;
        for index in self.tiles() {
            (current_width, current_height) = self._compute_tile_size(index, current_width, current_height);
        }
    }

    fn _compute_tile_size(&self, i: usize, c_w: f32, c_h: f32) -> (u16, u16) {
        let image = self.tile_handler.tile(i);
        let (max_tile_width, max_tile_height) = self.tile_size_hint();
        // Vec<(scale, width, height)>
        let (_scale, width, height) = self.compute_size(
            image,
            max_tile_width as f32,
            max_tile_height as f32,
        );
        let mut tile_width = self.tile_width;
        if width > c_w {
            tile_width = min(width as u16, self.max_tile_width);
        }
        let mut tile_height = self.tile_height;
        if height > c_h {
            tile_height = min(height as u16, self.max_tile_height);
        }
        (tile_width, tile_height)
    }

    fn tiles(&self) -> &Vec<usize>;

    fn tile(&self, i: usize) -> &Texture;

    fn act(&self, _i: usize) -> TileAction {
        TileAction::None
    }

    fn highlight_color(&self, _i: usize) -> Color {
        [1.0, 1.0, 1.0, 1.0]
    }

    fn background_color(&self) -> Color {
        [0.1, 0.2, 0.3, 1.0]
    }

    fn key_down(
        &mut self,
        _i: usize,
        keycode: Key,
        keymod: ModifierKey,
    ) -> Option<(Key, ModifierKey)> {
        return Some((keycode, keymod));
    }
}

// margin: the total space between items the grid
pub struct Grid<'a> {
    pub tile_handler: Box<&'a mut dyn TileHandler>,
    margin: usize,
    max_tile_width: u16,
    max_tile_height: u16,
    tile_width: u16,
    tile_height: u16,
    border_margin: usize,
    selected_tile: usize,
    highlight_border: usize,
    tiles_per_row: usize,
    margin_to_center: usize,
    coords_to_select: Option<(f32, f32)>,
    draw_tile: bool,
    pub allow_draw_tile: bool,
    width: f64,
    scroll_pos: f64,
    mouse_pos: [f64; 2],
}

impl<'a> Grid<'a> {
    pub fn new(
        tile_handler: Box<&'a mut dyn TileHandler>,
        tile_width: u16,
        tile_height: u16,
    ) -> Grid {
        let mut grid = Grid {
            tile_handler,
            margin: 5,
            max_tile_width: tile_width,
            max_tile_height: tile_height,
            tile_width: 0,
            tile_height: 0,
            border_margin: 20,
            selected_tile: 0,
            highlight_border: 2,
            tiles_per_row: 10,
            margin_to_center: 0,
            coords_to_select: None,
            draw_tile: false,
            allow_draw_tile: true,
            width: 0.0,
            scroll_pos: 0.0,
            mouse_pos: [0.0, 0.0],
        };
        let mut tmp_tile_width = tile_width;
        let mut tmp_tile_height = tile_height;
        for i in grid.tile_handler.tiles() {
            let result = grid.compute_max_size(*i, tmp_tile_width as f32, tmp_tile_height as f32);
            tmp_tile_width = result.0;
            tmp_tile_height = result.1;
        }
        grid.tile_width = tmp_tile_width;
        grid.tile_height = tmp_tile_height;
        grid
    }

    fn resize(&mut self, new_width: f32) {
        let remaining_width = new_width as usize - self.border_margin * 2 + self.margin;
        let tile_margin_width = self.tile_width as usize + self.margin;
        // TODO: tiles per row and margin to center do not handle case where
        // tiles per row is greater than the number of tiles to display which
        // leads to a not fully centered grid since all tiles can fit on a single
        // row.
        self.tiles_per_row = remaining_width / tile_margin_width;
        self.margin_to_center = remaining_width % tile_margin_width / 2;
        // minimum tiles per row is 1 regardless of window size
        if self.tiles_per_row == 0 {
            self.tiles_per_row = 1;
        }
    }

    fn up(&mut self) {
        self.selected_tile = max(
            0 as isize,
            self.selected_tile as isize - self.tiles_per_row as isize,
        ) as usize;
    }

    fn down(&mut self) {
        self.selected_tile = min(
            self.tile_handler.tiles().len() - 1,
            self.selected_tile + self.tiles_per_row,
        );
    }

    fn left(&mut self) {
        self.selected_tile = max(0 as isize, self.selected_tile as isize - 1) as usize;
    }

    fn right(&mut self) {
        self.selected_tile = min(self.tile_handler.tiles().len() - 1, self.selected_tile + 1);
    }

    fn select_tile_under(&mut self, x: f32, y: f32) {
        self.coords_to_select = Some((x, y));
    }

    pub fn run(&mut self, window: &mut Window, gl: &mut GlGraphics) -> Result<(), Error> {
        let mut settings = EventSettings::new();
        settings.set_lazy(true);
        settings.swap_buffers(true);
        settings.max_fps(1);
        settings.ups(1);
        let mut events = Events::new(settings);
        let mut modkeys = ModifierKey::NO_MODIFIER;
        while let Some(e) = events.next(window) {
            if let Some(r) = e.render_args() {
                self.draw(gl, &r)?;
            }

            if let Some(_u) = e.update_args() {
                self.update(gl)?;
            }

            if let Some(pos) = e.mouse_cursor_args() {
                self.mouse_pos = pos;
            }

            if let Some(scroll) = e.mouse_scroll_args() {
                self.mouse_wheel_event(scroll[0] as f32, scroll[1] as f32);
            }

            if let Some(p) = e.release_args() {
                match p {
                    Button::Mouse(button) => {
                        self.mouse_button_up_event(
                            button,
                            self.mouse_pos[0] as f32,
                            self.mouse_pos[1] as f32,
                        );
                        window.set_title(self.tile_handler.window_title());
                    }
                    _ => {}
                }
            }

            modkeys.event(&e);

            if let Some(p) = e.press_args() {
                match p {
                    Button::Keyboard(key) => {
                        self.key_down_event(key, modkeys, false);
                        window.set_title(self.tile_handler.window_title());
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
}

/*impl Widget for Grid<'_> {
    fn next(&self) -> NextAction {
        return NextAction::None;
    }
}*/

pub type GridResult<T> = Result<T, Error>;

pub trait EventHandler {
    fn update(&mut self, gl: &mut GlGraphics) -> GridResult<()>;
    fn draw(&mut self, gl: &mut GlGraphics, args: &RenderArgs) -> GridResult<()>;
    fn key_down_event(&mut self, keycode: Key, keymod: ModifierKey, repeat: bool);
    fn key_up_event(&mut self, keycode: Key, keymod: ModifierKey);
    fn mouse_button_up_event(&mut self, button: MouseButton, x: f32, y: f32);
    fn mouse_wheel_event(&mut self, x: f32, y: f32);
}

impl EventHandler for Grid<'_> {
    fn update(&mut self, _gl: &mut GlGraphics) -> GridResult<()> {
        const DESIRED_FPS: u32 = 20;
        //while timer::check_update_time(ctx, DESIRED_FPS) {
        //println!("Delta frame time: {:?} ", timer::delta(ctx));
        //println!("Average FPS: {}", timer::fps(ctx));
        //thread::sleep(time::Duration::from_millis(1000 / 40));
        //}
        Ok(())
    }

    fn draw(&mut self, gl: &mut GlGraphics, args: &RenderArgs) -> GridResult<()> {
        // handle window resize
        let [win_width, win_height] = args.window_size;
        if win_width != self.width {
            self.resize(win_width as f32);
            self.width = win_width;
        }
        let viewport = args.viewport();

        // clear the screen
        gl.draw(viewport, |_c, gl| {
            use graphics::clear;
            clear(self.tile_handler.background_color(), gl);
        });
        let mut x;
        let mut y = self.border_margin as f32;
        let mut start_at = (viewport.rect[1] as usize)
            / (self.tile_height as usize + self.margin) as usize
            * self.tiles_per_row;
        let row_of_selection: usize = self.selected_tile / self.tiles_per_row;
        if start_at > row_of_selection {
            start_at = row_of_selection;
        }
        let mut move_win_by = 0.0;
        let tiles = self.tile_handler.tiles();
        if self.selected_tile >= tiles.len() {
            self.selected_tile = tiles.len() - 1;
        }
        let mut launch = false;
        let mut tile_width = self.tile_width;
        let mut tile_height = self.tile_height;
        for (i, ii) in tiles.iter().enumerate() {
            let result = self.compute_max_size(*ii, tile_width as f32, tile_height as f32);
            tile_width = result.0;
            tile_height = result.1;
            let image = self.tile_handler.tile(*ii);
            let (scale, width, height) =
                Grid::compute_size(image, self.tile_width as f32, self.tile_height as f32);
            x = (self.margin_to_center
                + self.border_margin
                + i % self.tiles_per_row * self.tile_width as usize
                + i % self.tiles_per_row * self.margin) as f32;

            // Increment y when we start a new row
            if i != 0 && i % self.tiles_per_row == 0 {
                y += (self.margin + self.tile_height as usize) as f32;

                // Optimization to only draw a single page of images
                if i > self.selected_tile && y > (self.scroll_pos as f32 + win_height as f32) {
                    break;
                }
            }

            // Handle mouse selection of tiles
            if let Some((x_coord, y_coord)) = self.coords_to_select {
                if x_coord >= x
                    && x_coord <= x + self.tile_width as f32
                    && y_coord >= y
                    && y_coord <= y + self.tile_height as f32
                {
                    if self.selected_tile == i {
                        launch = true;
                    }
                    self.selected_tile = i;
                    self.coords_to_select = None;
                }
            }

            // Skip to next tile if current tile is offscreen
            if i < start_at {
                continue;
            }

            let x_image_margin = (self.tile_width as f32 - width) / 2.0;
            let y_image_margin = (self.tile_height as f32 - height) / 2.0;

            // Draw current tile
            gl.draw(viewport, |c, gl| {
                let transform = c
                    .transform
                    .trans((x + x_image_margin) as f64, (y + y_image_margin) as f64)
                    .trans(0.0, -self.scroll_pos)
                    .zoom(scale.into());
                let state = DrawState::default();
                Image::new().draw(image, &state, transform, gl);
            });

            // Draw outline around selected tile
            if i == self.selected_tile {
                gl.draw(viewport, |c, gl| {
                    let rect = graphics::rectangle::Rectangle::new_border(
                        self.tile_handler.highlight_color(*ii),
                        self.highlight_border as f64,
                    );
                    let transform = c.transform.trans(0.0, 0.0).trans(0.0, -self.scroll_pos);
                    rect.draw(
                        [
                            (x + x_image_margin) as f64,
                            (y + y_image_margin) as f64,
                            width as f64,
                            height as f64,
                        ],
                        &Default::default(),
                        transform,
                        gl,
                    );
                });

                // See if the window needs to be scrolled
                if y as f64 + self.tile_height as f64 > self.scroll_pos + win_height as f64 {
                    move_win_by = height;
                }
                if (y as f64) < self.scroll_pos {
                    move_win_by = -height;
                }
            }
        }
        self.tile_width = tile_width;
        self.tile_height = tile_height;

        // Trigger action if tile was clicked
        if launch {
            self.key_down_event(Key::Return, ModifierKey::NO_MODIFIER, false);
        }

        // Draw current image full screen
        if self.draw_tile {
            // TODO: Move into Tile trait
            let image = &self
                .tile_handler
                .tile(self.tile_handler.tiles()[self.selected_tile]);
            let (width, height) = image.get_size();
            let scale = f64::min(win_width / width as f64, win_height / height as f64);
            let width = width as f64 * scale;
            let height = height as f64 * scale;
            let x = (win_width - width) / 2.0;
            let y = (win_height - height) / 2.0;

            // draw overlay and image
            gl.draw(viewport, |c, gl| {
                let rect = graphics::rectangle::Rectangle::new([1.0, 1.0, 1.0, 1.0]);
                let transform = c.transform.trans(0.0, 0.0);
                rect.draw(
                    [0.0, 0.0, win_width, win_height],
                    &Default::default(),
                    transform,
                    gl,
                );

                let transform = c.transform.trans(x as f64, y as f64).zoom(scale.into());
                let state = DrawState::default();
                Image::new().draw(*image, &state, transform, gl);
            });
        }
        if move_win_by != 0.0 {
            self.scroll_pos += move_win_by as f64;
        }

        Ok(())
    }

    fn mouse_button_up_event(&mut self, _button: MouseButton, x: f32, y: f32) {
        self.select_tile_under(x, y + self.scroll_pos as f32);
    }

    fn mouse_wheel_event(&mut self, _x: f32, y: f32) {
        if y > 0.0 {
            if self.draw_tile {
                self.left();
                return;
            }
            self.up();
        }
        if y < 0.0 {
            if self.draw_tile {
                self.right();
                return;
            }
            self.down();
        }
    }

    fn key_down_event(&mut self, keycode: Key, keymod: ModifierKey, _repeat: bool) {
        let result = self
            .tile_handler
            .key_down(self.selected_tile, keycode, keymod);
        if let None = result {
            return;
        }
        let (keycode, keymod) = result.unwrap();
        match keycode {
            Key::E => if keymod.contains(ModifierKey::CTRL) {},
            Key::Up => {
                self.up();
            }
            Key::Down => {
                self.down();
            }
            Key::Left => {
                self.left();
            }
            Key::Right => {
                self.right();
            }
            Key::Return => {
                if self.allow_draw_tile && !self.draw_tile {
                    self.draw_tile = true;
                } else {
                    self.tile_handler
                        .act(self.tile_handler.tiles()[self.selected_tile]);
                }
            }
            Key::Home => {
                self.selected_tile = 0;
            }
            Key::End => {
                self.selected_tile = self.tile_handler.tiles().len() - 1;
            }
            Key::Escape => {
                if self.draw_tile {
                    self.draw_tile = false;
                } else {
                    //ggez::event::quit(ctx);
                }
            }
            _ => {
                return ();
            }
        }
    }

    fn key_up_event(&mut self, keycode: Key, _keymod: ModifierKey) {
        match keycode {
            _ => {
                return ();
            }
        }
    }
}
