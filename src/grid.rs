use failure::Error;
use glutin_window::GlutinWindow as Window;
use graphics::{DrawState, Image, ImageSize, Transformed};
use log::trace;
use opengl_graphics::{GlGraphics, Texture};
use piston::event_loop::*;
use piston::input::{
    keyboard::{Key, ModifierKey},
    mouse::MouseButton,
    Button, MouseCursorEvent, MouseScrollEvent, PressEvent, ReleaseEvent, RenderArgs, RenderEvent,
};
use piston::window::AdvancedWindow;
use std::cmp::{max, min};

pub type Color = [f32; 4];
pub type GridResult<T> = Result<T, Error>;

pub trait GridModel {
    fn window_title(&self) -> String;
    fn tiles(&self) -> &Vec<usize>;
    fn tile(&self, i: usize) -> &Texture;
    fn act(&self, i: usize);
}

pub struct GridController {
    model: Box<dyn GridModel>,
    selected_tile: usize,
    coords_to_select: Option<(f64, f64)>,
    pub allow_draw_tile: bool,
    draw_tile: bool,
    mouse_pos: [f64; 2],
    tiles_per_row: usize,
}

impl GridController {
    pub fn new(model: Box<dyn GridModel>) -> GridController {
        GridController {
            tiles_per_row: 10,
            model,
            selected_tile: 0,
            coords_to_select: None,
            allow_draw_tile: true,
            draw_tile: false,
            mouse_pos: [0.0, 0.0],
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
            self.model.tiles().len() - 1,
            self.selected_tile + self.tiles_per_row,
        );
    }

    fn left(&mut self) {
        self.selected_tile = max(0 as isize, self.selected_tile as isize - 1) as usize;
    }

    fn right(&mut self) {
        self.selected_tile = min(self.model.tiles().len() - 1, self.selected_tile + 1);
    }

    fn select_tile_under(&mut self, x: f64, y: f64) {
        self.coords_to_select = Some((x, y));
    }

    fn mouse_button_up_event(&mut self, _button: MouseButton, x: f64, y: f64) {
        self.select_tile_under(x, y);
    }

    fn mouse_wheel_event(&mut self, _x: f64, y: f64) {
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
        // let result = self
        //     .tile_handler
        //     .key_down(self.selected_tile, keycode, keymod);
        // if let None = result {
        //     return;
        // }
        // let (keycode, keymod) = result.unwrap();
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
                    self.model.act(self.model.tiles()[self.selected_tile]);
                }
            }
            Key::Home => {
                self.selected_tile = 0;
            }
            Key::End => {
                self.selected_tile = self.model.tiles().len() - 1;
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

pub struct GridView {
    tile_width: usize,
    tile_height: usize,
    margin: usize,
    margin_to_center: usize,
    border_margin: usize,
    scroll_pos: f64,
    width: f64,
    tile_size_hint: (usize, usize),
    highlight_border: f64,
}

impl GridView {
    pub fn new(tile_width: usize, tile_height: usize) -> GridView {
        GridView {
            tile_width: 0,
            tile_height: 0,
            margin: 5,
            margin_to_center: 0,
            border_margin: 20,
            scroll_pos: 0.0,
            width: 0.0,
            tile_size_hint: (tile_width, tile_height),
            highlight_border: 2.0,
        }
    }

    fn compute_size(
        &self,
        c: &GridController,
        i: usize,
        w: usize,
        h: usize,
    ) -> (f64, usize, usize) {
        let (width, height) = c.model.tile(i).get_size();
        let scale = f64::min(w as f64 / width as f64, h as f64 / height as f64);
        let width = width as f64 * scale;
        let height = height as f64 * scale;
        (scale, width as usize, height as usize)
    }

    fn compute_tile_size(&self, c: &GridController) -> (usize, usize) {
        let mut current_size = (0, 0);
        for index in c.model.tiles() {
            current_size = self._compute_tile_size(c, *index, current_size.0, current_size.1);
        }
        //println!("{:?} {:?}", current_size, self.tile_size_hint);
        current_size
    }

    fn _compute_tile_size(
        &self,
        c: &GridController,
        i: usize,
        c_w: usize,
        c_h: usize,
    ) -> (usize, usize) {
        let (max_tile_width, max_tile_height) = self.tile_size_hint;
        // Vec<(scale, width, height)>
        let (_scale, width, height) = self.compute_size(c, i, max_tile_width, max_tile_height);
        //println!("{} {} {} {} {}", _scale, width, height, c_w, c_h);
        let mut tile_width = c_w;
        if width > c_w {
            tile_width = min(width, max_tile_width);
        }
        let mut tile_height = c_h;
        if height > c_h {
            tile_height = min(height, max_tile_height);
        }
        (tile_width, tile_height)
    }

    fn resize(&mut self, new_width: f64) -> usize {
        let remaining_width = new_width as usize - self.border_margin * 2 + self.margin;
        let tile_margin_width = self.tile_width as usize + self.margin;
        // TODO: tiles per row and margin to center do not handle case where
        // tiles per row is greater than the number of tiles to display which
        // leads to a not fully centered grid since all tiles can fit on a single
        // row.
        let mut tiles_per_row = remaining_width / tile_margin_width;
        self.margin_to_center = remaining_width % tile_margin_width / 2;
        // minimum tiles per row is 1 regardless of window size
        if tiles_per_row == 0 {
            tiles_per_row = 1;
        }
        trace!("Tiles per row: {}", tiles_per_row);
        tiles_per_row
    }

    fn draw(
        &mut self,
        gl: &mut GlGraphics,
        args: &RenderArgs,
        c: &mut GridController,
    ) -> GridResult<()> {
        let size = self.compute_tile_size(c);
        self.tile_width = size.0;
        self.tile_height = size.1;
        //println!("{} {}", self.tile_width, self.tile_height);
        // handle window resize
        let [win_width, win_height] = args.window_size;
        if win_width != self.width {
            c.tiles_per_row = self.resize(win_width);
            self.width = win_width;
        }
        let viewport = args.viewport();

        // clear the screen
        gl.draw(viewport, |_c, gl| {
            use graphics::clear;
            clear(self.background_color(), gl);
        });
        let mut x: f64;
        let mut y: f64 = self.border_margin as f64;
        let mut start_at = (viewport.rect[1] as usize)
            / (self.tile_height as usize + self.margin) as usize
            * c.tiles_per_row;
        let row_of_selection: usize = c.selected_tile / c.tiles_per_row;
        if start_at > row_of_selection {
            start_at = row_of_selection;
        }
        let mut move_win_by = 0.0;
        let tiles = c.model.tiles();
        if c.selected_tile >= tiles.len() {
            c.selected_tile = tiles.len() - 1;
        }
        let mut launch = false;
        for (i, ii) in tiles.iter().enumerate() {
            let image = c.model.tile(*ii);
            let (scale, width, height) =
                self.compute_size(c, *ii, self.tile_width, self.tile_height);
            x = (self.margin_to_center
                + self.border_margin
                + i % c.tiles_per_row * self.tile_width
                + i % c.tiles_per_row * self.margin) as f64;

            // Increment y when we start a new row
            if i != 0 && i % c.tiles_per_row == 0 {
                y += (self.margin + self.tile_height) as f64;

                // Optimization to only draw a single page of images
                if i > c.selected_tile && y > (self.scroll_pos + win_height) as f64 {
                    break;
                }
            }

            // Handle mouse selection of tiles
            if let Some((x_coord, y_coord)) = c.coords_to_select {
                if x_coord >= x
                    && x_coord <= x + self.tile_width as f64
                    && y_coord >= y + self.scroll_pos
                    && y_coord <= y + self.scroll_pos + self.tile_height as f64
                {
                    if c.selected_tile == i {
                        launch = true;
                    }
                    c.selected_tile = i;
                    c.coords_to_select = None;
                }
            }

            // Skip to next tile if current tile is offscreen
            if i < start_at {
                continue;
            }

            let x_image_margin = (self.tile_width - width) / 2;
            let y_image_margin = (self.tile_height - height) / 2;

            // Draw current tile
            gl.draw(viewport, |c, gl| {
                let transform = c
                    .transform
                    .trans(x + x_image_margin as f64, y + y_image_margin as f64)
                    .trans(0.0, -self.scroll_pos)
                    .zoom(scale.into());
                let state = DrawState::default();
                Image::new().draw(image, &state, transform, gl);
            });

            // Draw outline around selected tile
            if i == c.selected_tile {
                gl.draw(viewport, |c, gl| {
                    let rect = graphics::rectangle::Rectangle::new_border(
                        self.highlight_color(*ii),
                        self.highlight_border as f64,
                    );
                    let transform = c.transform.trans(0.0, 0.0).trans(0.0, -self.scroll_pos);
                    rect.draw(
                        [
                            x + x_image_margin as f64,
                            y + y_image_margin as f64,
                            width as f64,
                            height as f64,
                        ],
                        &Default::default(),
                        transform,
                        gl,
                    );
                });

                // See if the window needs to be scrolled
                if y + self.tile_height as f64 > self.scroll_pos + win_height as f64 {
                    move_win_by = height as f64;
                }
                if (y as f64) < self.scroll_pos {
                    move_win_by = -(height as f64);
                }
            }
        }

        // Trigger action if tile was clicked
        if launch {
            c.key_down_event(Key::Return, ModifierKey::NO_MODIFIER, false);
        }

        // Draw current image full screen
        if c.draw_tile {
            // TODO: Move into Tile trait
            let image = &c.model.tile(c.model.tiles()[c.selected_tile]);
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

    fn highlight_color(&self, _i: usize) -> Color {
        [1.0, 1.0, 1.0, 1.0]
    }

    fn background_color(&self) -> Color {
        [0.1, 0.2, 0.3, 1.0]
    }
}

pub fn run(
    window: &mut Window,
    gl: &mut GlGraphics,
    controller: &mut GridController,
    view: &mut GridView,
) -> Result<(), Error> {
    let mut settings = EventSettings::new();
    settings.set_lazy(true);
    settings.swap_buffers(true);
    settings.max_fps(1);
    settings.ups(1);
    let mut events = Events::new(settings);
    let mut modkeys = ModifierKey::NO_MODIFIER;
    while let Some(e) = events.next(window) {
        if let Some(r) = e.render_args() {
            view.draw(gl, &r, controller)?;
        }

        if let Some(pos) = e.mouse_cursor_args() {
            controller.mouse_pos = pos;
        }

        if let Some(scroll) = e.mouse_scroll_args() {
            controller.mouse_wheel_event(scroll[0], scroll[1]);
        }

        if let Some(p) = e.release_args() {
            match p {
                Button::Mouse(button) => {
                    controller.mouse_button_up_event(
                        button,
                        controller.mouse_pos[0],
                        controller.mouse_pos[1],
                    );
                    window.set_title(controller.model.window_title());
                }
                _ => {}
            }
        }

        modkeys.event(&e);

        if let Some(p) = e.press_args() {
            match p {
                Button::Keyboard(key) => {
                    controller.key_down_event(key, modkeys, false);
                    window.set_title(controller.model.window_title());
                }
                _ => {}
            }
        }
    }
    Ok(())
}
