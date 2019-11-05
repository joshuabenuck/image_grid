use crate::dispatcher::{NextAction, Widget};
use failure::Error;
use ggez::event::{self, KeyCode, KeyMods, MouseButton};
use ggez::{self, graphics, timer, Context, GameResult};
use std::cmp::{max, min};
use std::process::Child;
use std::thread;
use std::time;

pub enum TileAction {
    None,
    Launch(Result<Child, Error>),
}

pub trait TileHandler {
    fn tiles(&self) -> &Vec<usize>;

    fn tile(&self, i: usize) -> &graphics::Image;

    fn act(&self, _i: usize) -> TileAction {
        TileAction::None
    }

    fn highlight_color(&self, _i: usize) -> graphics::Color {
        graphics::WHITE
    }

    fn background_color(&self) -> graphics::Color {
        [0.1, 0.2, 0.3, 1.0].into()
    }

    fn key_down(
        &mut self,
        _i: usize,
        keycode: KeyCode,
        keymod: KeyMods,
    ) -> Option<(KeyCode, KeyMods)> {
        return Some((keycode, keymod));
    }
}

// margin: the total space between items the grid
pub struct Grid<'a> {
    pub tile_handler: Box<&'a mut dyn TileHandler>,
    margin: usize,
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
    dirty: bool,
}

impl<'a> Grid<'a> {
    pub fn new(
        tile_handler: Box<&'a mut dyn TileHandler>,
        tile_width: u16,
        tile_height: u16,
    ) -> Grid {
        let images = tile_handler.tiles().iter().map(|i| tile_handler.tile(*i));
        // Vec<(scale, width, height)>
        let sizes: Vec<(f32, f32, f32)> = images
            .map(|i| Grid::compute_size(i, tile_width as f32, tile_height as f32))
            .collect();
        let max_width = (&sizes).iter().map(|size| size.1).fold(0.0, f32::max) as u16;
        let max_height = (&sizes).iter().map(|size| size.2).fold(0.0, f32::max) as u16;
        let tile_width = min(max_width, tile_width);
        let tile_height = min(max_height, tile_height);
        Grid {
            tile_handler,
            margin: 5,
            tile_width,
            tile_height,
            border_margin: 20,
            selected_tile: 0,
            highlight_border: 2,
            tiles_per_row: 0,
            margin_to_center: 0,
            coords_to_select: None,
            draw_tile: false,
            allow_draw_tile: true,
            dirty: true,
        }
    }

    fn resize(&mut self, new_width: f32) {
        self.dirty = true;
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
        println!("tiles_per_row: {}", self.tiles_per_row);
    }

    fn compute_tile_size() {}

    fn scroll_by() {}

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

    fn compute_size(image: &graphics::Image, w: f32, h: f32) -> (f32, f32, f32) {
        let scale = f32::min(w / image.width() as f32, h / image.height() as f32);
        let width = image.width() as f32 * scale;
        let height = image.height() as f32 * scale;
        (scale, width, height)
    }
}

impl Widget for Grid<'_> {
    fn next(&self) -> NextAction {
        return NextAction::None;
    }
}

impl event::EventHandler for Grid<'_> {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        const DESIRED_FPS: u32 = 20;
        while timer::check_update_time(ctx, DESIRED_FPS) {
            //println!("Delta frame time: {:?} ", timer::delta(ctx));
            //println!("Average FPS: {}", timer::fps(ctx));
            thread::sleep(time::Duration::from_millis(1000 / 40));
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        if !self.dirty {
            return Ok(());
        }
        graphics::clear(ctx, self.tile_handler.background_color());
        let mut x;
        let mut y = self.border_margin as f32;
        let mut screen = graphics::screen_coordinates(ctx);
        let mut start_at = (screen.y as usize) / (self.tile_height as usize + self.margin) as usize
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
        for (i, ii) in tiles.iter().enumerate() {
            let image = self.tile_handler.tile(*ii);
            let (scale, width, height) =
                Grid::compute_size(image, self.tile_width as f32, self.tile_height as f32);
            x = (self.margin_to_center
                + self.border_margin
                + i % self.tiles_per_row * self.tile_width as usize
                + i % self.tiles_per_row * self.margin) as f32;
            if i != 0 && i % self.tiles_per_row == 0 {
                y += (self.margin + self.tile_height as usize) as f32;
                if i > self.selected_tile && y > (screen.y + screen.h) {
                    break;
                }
            }
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
            if i < start_at {
                continue;
            }
            let x_image_margin = (self.tile_width as f32 - width) / 2.0;
            let y_image_margin = (self.tile_height as f32 - height) / 2.0;
            let dest_point = mint::Point2 {
                x: x + x_image_margin,
                y: y + y_image_margin,
            };
            graphics::draw(
                ctx,
                image,
                graphics::DrawParam::default()
                    .dest(dest_point)
                    .scale([scale, scale]),
            )?;
            if i == self.selected_tile {
                let rectangle = graphics::Mesh::new_rectangle(
                    ctx,
                    graphics::DrawMode::stroke(self.highlight_border as f32),
                    graphics::Rect::new(x + x_image_margin, y + y_image_margin, width, height),
                    self.tile_handler.highlight_color(*ii),
                )?;
                graphics::draw(ctx, &rectangle, (ggez::nalgebra::Point2::new(0.0, 0.0),))?;
                if y + self.tile_height as f32 > screen.y + screen.h {
                    move_win_by = height;
                }
                if y < screen.y {
                    move_win_by = -height;
                }
            }
        }
        if launch {
            self.key_down_event(ctx, KeyCode::Return, KeyMods::NONE, false);
        }
        if self.draw_tile {
            // draw overlay
            let rectangle = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                graphics::Rect::new(0.0, screen.y, screen.w, screen.h),
                graphics::Color::from([1.0, 1.0, 1.0, 1.0]),
            )?;
            graphics::draw(ctx, &rectangle, (ggez::nalgebra::Point2::new(0.0, 0.0),))?;

            // draw currently selected image
            // TODO: Move into Tile trait
            let image = &self
                .tile_handler
                .tile(self.tile_handler.tiles()[self.selected_tile]);
            let scale = f32::min(
                screen.w / image.width() as f32,
                screen.h / image.height() as f32,
            );
            let width = image.width() as f32 * scale;
            let height = image.height() as f32 * scale;
            let x = (screen.w - width) / 2.0;
            let y = (screen.h - height) / 2.0 + screen.y;
            let dest_point = mint::Point2 { x, y };
            graphics::draw(
                ctx,
                *image,
                graphics::DrawParam::default()
                    .dest(dest_point)
                    .scale([scale, scale]),
            )?;
        }
        self.dirty = false;
        if move_win_by != 0.0 {
            screen.y += move_win_by;
            graphics::set_screen_coordinates(ctx, screen)?;
            self.dirty = true;
        }

        graphics::present(ctx)?;
        timer::yield_now();
        Ok(())
    }

    fn mouse_button_up_event(&mut self, ctx: &mut Context, _button: MouseButton, x: f32, y: f32) {
        let screen = graphics::screen_coordinates(ctx);
        self.select_tile_under(x, y + screen.y);
        self.dirty = true;
    }

    fn mouse_wheel_event(&mut self, _ctx: &mut Context, _x: f32, y: f32) {
        if y > 0.0 {
            if self.draw_tile {
                self.left();
                self.dirty = true;
                return;
            }
            self.up();
        }
        if y < 0.0 {
            if self.draw_tile {
                self.right();
                self.dirty = true;
                return;
            }
            self.down();
        }
        self.dirty = true;
    }

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: KeyCode,
        keymod: KeyMods,
        _repeat: bool,
    ) {
        let result = self
            .tile_handler
            .key_down(self.selected_tile, keycode, keymod);
        if let None = result {
            self.dirty = true;
            return;
        }
        let (keycode, keymod) = result.unwrap();
        match keycode {
            KeyCode::E => if keymod.contains(KeyMods::CTRL) {},
            KeyCode::Up => {
                self.up();
            }
            KeyCode::Down => {
                self.down();
            }
            KeyCode::Left => {
                self.left();
            }
            KeyCode::Right => {
                self.right();
            }
            KeyCode::Return => {
                if self.allow_draw_tile && !self.draw_tile {
                    self.draw_tile = true;
                } else {
                    self.tile_handler
                        .act(self.tile_handler.tiles()[self.selected_tile]);
                }
            }
            KeyCode::Home => {
                self.selected_tile = 0;
            }
            KeyCode::End => {
                self.selected_tile = self.tile_handler.tiles().len() - 1;
            }
            KeyCode::Escape => {
                if self.draw_tile {
                    self.draw_tile = false;
                    self.dirty = true;
                } else {
                    ggez::event::quit(ctx);
                }
            }
            _ => {
                return ();
            }
        }
        self.dirty = true;
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymod: KeyMods) {
        match keycode {
            _ => {
                return ();
            }
        }
    }

    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) {
        println!("Resized screen to {}, {}", width, height);
        let mut screen = graphics::screen_coordinates(ctx);
        screen.w = width as f32;
        screen.h = height as f32;
        graphics::set_screen_coordinates(ctx, screen).unwrap();
        self.resize(screen.w);
    }
}
