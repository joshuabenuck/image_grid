use anyhow::Error;
use glutin_window::GlutinWindow as Window;
use graphics::math::Matrix2d;
use graphics::{DrawState, Image, ImageSize, Transformed};
use opengl_graphics::{GlGraphics, Texture};
use piston::event_loop::*;
use piston::input::{
    keyboard::{Key, ModifierKey},
    mouse::MouseButton,
    Button, MouseCursorEvent, MouseScrollEvent, PressEvent, ReleaseEvent, RenderArgs, RenderEvent,
};
use piston::window::AdvancedWindow;
use std::cmp::{max, min};

pub type GridResult<T> = Result<T, Error>;

pub type Color = [f32; 4];

pub trait TileHandler {
    fn window_title(&self) -> String;

    fn tiles(&self) -> &Vec<usize>;

    fn tile(&self, i: usize) -> &Texture;

    fn act(&mut self, _i: usize) {}

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

    fn key_up(
        &mut self,
        _i: usize,
        keycode: Key,
        keymod: ModifierKey,
    ) -> Option<(Key, ModifierKey)> {
        return Some((keycode, keymod));
    }

    fn compute_size_by_index(&self, i: usize, w: usize, h: usize) -> (f64, usize, usize) {
        self.compute_size(self.tile(i), w, h)
    }

    fn compute_size(&self, image: &Texture, w: usize, h: usize) -> (f64, usize, usize) {
        let (width, height) = image.get_size();
        let scale = f64::min(w as f64 / width as f64, h as f64 / height as f64);
        let width = width as f64 * scale;
        let height = height as f64 * scale;
        (scale, width as usize, height as usize)
    }

    fn draw_tile(
        &self,
        i: usize,
        transform: Matrix2d,
        gl: &mut GlGraphics,
        target_width: usize,
        target_height: usize,
    ) {
        let image = self.tile(i);
        let (scale, width, height) = self.compute_size_by_index(i, target_width, target_height);
        let x_image_margin = (target_width - width) / 2;
        let y_image_margin = (target_height - height) / 2;

        let state = DrawState::default();
        Image::new().draw(
            image,
            &state,
            transform
                .trans(x_image_margin as f64, y_image_margin as f64)
                .zoom(scale.into()),
            gl,
        );
    }

    fn draw_outline(
        &self,
        i: usize,
        transform: Matrix2d,
        gl: &mut GlGraphics,
        target_width: usize,
        target_height: usize,
    ) {
        let (_scale, width, height) = self.compute_size_by_index(i, target_width, target_height);
        let x_image_margin = (target_width - width) / 2;
        let y_image_margin = (target_height - height) / 2;
        let rect = graphics::rectangle::Rectangle::new_border(self.highlight_color(i), 2.0);
        rect.draw(
            [
                x_image_margin as f64,
                y_image_margin as f64,
                width as f64,
                height as f64,
            ],
            &Default::default(),
            transform,
            gl,
        );
    }
}

// margin: the total space between items the grid
pub struct Grid<'a> {
    pub tile_handler: Box<&'a mut dyn TileHandler>,
    margin: usize,
    tile_width: usize,
    tile_height: usize,
    border_margin: usize,
    selected_tile: usize,
    tiles_per_row: usize,
    margin_to_center: usize,
    coords_to_select: Option<(f64, f64)>,
    draw_tile: bool,
    pub allow_draw_tile: bool,
    width: f64,
    scroll_pos: f64,
    mouse_pos: [f64; 2],
}

impl<'a> Grid<'a> {
    pub fn new(
        tile_handler: Box<&'a mut dyn TileHandler>,
        tile_width: usize,
        tile_height: usize,
    ) -> Grid {
        // Vec<(scale, width, height)>
        let sizes: Vec<(f64, usize, usize)> = tile_handler
            .tiles()
            .iter()
            .map(|i| tile_handler.compute_size_by_index(*i, tile_width, tile_height))
            .collect();
        let max_width = (&sizes).iter().map(|size| size.1).fold(0, max) as usize;
        let max_height = (&sizes).iter().map(|size| size.2).fold(0, max) as usize;
        let tile_width = min(max_width, tile_width);
        let tile_height = min(max_height, tile_height);
        Grid {
            tile_handler,
            margin: 5,
            tile_width,
            tile_height,
            border_margin: 20,
            selected_tile: 0,
            tiles_per_row: 10,
            margin_to_center: 0,
            coords_to_select: None,
            draw_tile: false,
            allow_draw_tile: true,
            width: 0.0,
            scroll_pos: 0.0,
            mouse_pos: [0.0, 0.0],
        }
    }

    fn resize(&mut self, new_width: usize) {
        let remaining_width = new_width - self.border_margin * 2 + self.margin;
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

    fn select_tile_under(&mut self, x: f64, y: f64) {
        self.coords_to_select = Some((x, y));
    }

    pub fn run(&mut self, window: &mut Window, gl: &mut GlGraphics) -> Result<(), Error> {
        let mut settings = EventSettings::new();
        settings.set_lazy(false);
        settings.swap_buffers(true);
        settings.max_fps(1);
        settings.ups(1);
        let mut events = Events::new(settings);
        let mut modkeys = ModifierKey::NO_MODIFIER;
        loop {
            if let Some(e) = events.next(window) {
                if let Some(r) = e.render_args() {
                    self.draw(gl, &r)?;
                }

                if let Some(pos) = e.mouse_cursor_args() {
                    self.mouse_pos = pos;
                }

                if let Some(scroll) = e.mouse_scroll_args() {
                    self.mouse_wheel_event(scroll[0] as f32, scroll[1] as f32);
                }

                if let Some(p) = e.release_args() {
                    match p {
                        Button::Keyboard(key) => {
                            self.key_up_event(key, modkeys);
                        }
                        Button::Mouse(button) => {
                            self.mouse_button_up_event(
                                button,
                                self.mouse_pos[0],
                                self.mouse_pos[1],
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
            } else {
                break;
            }
        }
        Ok(())
    }

    fn draw(&mut self, gl: &mut GlGraphics, args: &RenderArgs) -> GridResult<()> {
        // handle window resize
        let [win_width, win_height] = args.window_size;
        if win_width != self.width {
            self.resize(win_width as usize);
            self.width = win_width;
        }
        let viewport = args.viewport();

        // clear the screen
        gl.draw(viewport, |_c, gl| {
            use graphics::clear;
            clear(self.tile_handler.background_color(), gl);
        });
        let mut x: f64;
        let mut y: f64 = self.border_margin as f64;
        let mut start_at =
            (viewport.rect[1] as usize) / (self.tile_height + self.margin) * self.tiles_per_row;
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

        for (i, ii) in tiles.iter().enumerate() {
            x = (self.margin_to_center
                + self.border_margin
                + i % self.tiles_per_row * self.tile_width
                + i % self.tiles_per_row * self.margin) as f64;

            // Increment y when we start a new row
            if i != 0 && i % self.tiles_per_row == 0 {
                y += (self.margin + self.tile_height) as f64;

                // Optimization to only draw a single page of images
                if i > self.selected_tile && y > (self.scroll_pos + win_height) as f64 {
                    break;
                }
            }

            // Handle mouse selection of tiles
            if let Some((x_coord, y_coord)) = self.coords_to_select {
                if x_coord >= x
                    && x_coord <= x + self.tile_width as f64
                    && y_coord >= y
                    && y_coord <= y + self.tile_height as f64
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

            // Draw current tile
            gl.draw(viewport, |c, gl| {
                let transform = c.transform.trans(x, y).trans(0.0, -self.scroll_pos);
                self.tile_handler
                    .draw_tile(*ii, transform, gl, self.tile_width, self.tile_height);
            });

            // Draw outline around selected tile
            if i == self.selected_tile {
                gl.draw(viewport, |c, gl| {
                    let transform = c.transform.trans(x, y).trans(0.0, -self.scroll_pos);
                    self.tile_handler.draw_outline(
                        *ii,
                        transform,
                        gl,
                        self.tile_width,
                        self.tile_height,
                    );
                });

                // See if the window needs to be scrolled
                if y + self.tile_height as f64 > self.scroll_pos + win_height as f64 {
                    move_win_by = self.tile_height as f64;
                }
                if (y as f64) < self.scroll_pos {
                    move_win_by = -(self.tile_height as f64);
                }
            }
        }

        // Trigger action if tile was clicked
        if launch {
            self.key_down_event(Key::Return, ModifierKey::NO_MODIFIER, false);
        }

        // Draw current image full screen
        if self.draw_tile {
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

                self.tile_handler.draw_tile(
                    self.tile_handler.tiles()[self.selected_tile],
                    c.transform,
                    gl,
                    win_width as usize,
                    win_height as usize,
                );
            });
        }
        if move_win_by != 0.0 {
            self.scroll_pos += move_win_by;
        }

        Ok(())
    }

    fn mouse_button_up_event(&mut self, _button: MouseButton, x: f64, y: f64) {
        self.select_tile_under(x, y + self.scroll_pos);
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

    fn key_up_event(&mut self, keycode: Key, keymod: ModifierKey) {
        let result = self
            .tile_handler
            .key_up(self.selected_tile, keycode, keymod);
        if let None = result {
            return;
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
}
