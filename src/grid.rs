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

pub trait Tile {
    fn image(&self) -> &graphics::Image;
    fn act(&self) -> TileAction {
        TileAction::None
    }
}

// margin: the total space between items the grid
pub struct Grid {
    tiles: Vec<Box<dyn Tile>>,
    margin: usize,
    tile_width: u16,
    tile_height: u16,
    border_margin: usize,
    selected_tile: isize,
    highlight_color: graphics::Color,
    highlight_border: usize,
    tiles_per_row: usize,
    margin_to_center: usize,
    coords_to_select: Option<(f32, f32)>,
    draw_tile: bool,
    dirty: bool,
}

impl Grid {
    pub fn new(tiles: Vec<Box<dyn Tile>>, tile_width: u16, tile_height: u16) -> Grid {
        let images: Vec<&graphics::Image> = tiles.iter().map(|t| t.image()).collect();
        // Vec<(scale, width, height)>
        let sizes: Vec<(f32, f32, f32)> = (&images)
            .iter()
            .map(|i| Grid::compute_size(i, tile_width as f32, tile_height as f32))
            .collect();
        let max_width = (&sizes).iter().map(|size| size.1).fold(0.0, f32::max) as u16;
        let max_height = (&sizes).iter().map(|size| size.2).fold(0.0, f32::max) as u16;
        let tile_width = min(max_width, tile_width);
        let tile_height = min(max_height, tile_height);
        Grid {
            tiles: tiles,
            margin: 5,
            tile_width,
            tile_height,
            border_margin: 20,
            selected_tile: 0,
            highlight_color: graphics::WHITE,
            highlight_border: 2,
            tiles_per_row: 0,
            margin_to_center: 0,
            coords_to_select: None,
            draw_tile: false,
            dirty: true,
        }
    }

    fn margin<'a>(&'a mut self, m: usize) -> &'a mut Self {
        self.margin = m;
        self
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
        self.selected_tile = max(0 as isize, self.selected_tile - self.tiles_per_row as isize);
    }

    fn down(&mut self) {
        self.selected_tile = min(
            (self.tiles.len() - 1) as isize,
            self.selected_tile + self.tiles_per_row as isize,
        );
    }

    fn left(&mut self) {
        self.selected_tile = max(0 as isize, self.selected_tile - 1);
    }

    fn right(&mut self) {
        self.selected_tile = min((self.tiles.len() - 1) as isize, self.selected_tile + 1);
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

impl Widget for Grid {
    fn next(&self) -> NextAction {
        return NextAction::None;
    }
}

impl event::EventHandler for Grid {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        const DESIRED_FPS: u32 = 20;
        while timer::check_update_time(ctx, DESIRED_FPS) {
            //println!("Delta frame time: {:?} ", timer::delta(ctx));
            //println!("Average FPS: {}", timer::fps(ctx));
            thread::sleep(time::Duration::from_millis(10));
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        if !self.dirty {
            return Ok(());
        }
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());
        let mut x;
        let mut y = self.border_margin as f32;
        let mut screen = graphics::screen_coordinates(ctx);
        let mut start_at = (screen.y as usize) / (self.tile_height as usize + self.margin) as usize
            * self.tiles_per_row;
        let row_of_selection: usize = self.selected_tile as usize / self.tiles_per_row;
        if start_at > row_of_selection {
            start_at = row_of_selection;
        }
        let mut move_win_by = 0.0;
        for (i, tile) in self.tiles.iter().enumerate() {
            let image = tile.image();
            let (scale, width, height) =
                Grid::compute_size(image, self.tile_width as f32, self.tile_height as f32);
            x = (self.margin_to_center
                + self.border_margin
                + i % self.tiles_per_row * self.tile_width as usize
                + i % self.tiles_per_row * self.margin) as f32;
            if i != 0 && i % self.tiles_per_row == 0 {
                y += (self.margin + self.tile_height as usize) as f32;
                if i > self.selected_tile as usize && y > (screen.y + screen.h) {
                    break;
                }
            }
            if let Some((x_coord, y_coord)) = self.coords_to_select {
                if x_coord >= x
                    && x_coord <= x + self.tile_width as f32
                    && y_coord >= y
                    && y_coord <= y + self.tile_height as f32
                {
                    self.selected_tile = i as isize;
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
            if i == self.selected_tile as usize {
                let rectangle = graphics::Mesh::new_rectangle(
                    ctx,
                    graphics::DrawMode::stroke(self.highlight_border as f32),
                    graphics::Rect::new(x + x_image_margin, y + y_image_margin, width, height),
                    self.highlight_color,
                )?;
                graphics::draw(ctx, &rectangle, (ggez::nalgebra::Point2::new(0.0, 0.0),))?;
                if y + self.tile_height as f32 > screen.y + screen.h {
                    move_win_by = height;
                }
                if y < screen.y {
                    move_win_by = -height;
                }
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
                let image = self.tiles[self.selected_tile as usize].image();
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
                    image,
                    graphics::DrawParam::default()
                        .dest(dest_point)
                        .scale([scale, scale]),
                )?;
            }
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
    }

    fn mouse_wheel_event(&mut self, _ctx: &mut Context, _x: f32, y: f32) {
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

    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: KeyCode,
        _keymod: KeyMods,
        _repeat: bool,
    ) {
        match keycode {
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
                if !self.draw_tile {
                    self.draw_tile = true;
                } else {
                    self.tiles[self.selected_tile as usize].act();
                }
            }
            KeyCode::Home => {
                self.selected_tile = 0;
            }
            KeyCode::End => {
                self.selected_tile = (self.tiles.len() - 1) as isize;
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
