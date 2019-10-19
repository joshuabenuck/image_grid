use clap::{App, Arg};
use ggez::event::{self, KeyCode, KeyMods};
use ggez::{self, filesystem, graphics, timer, Context, GameResult};
use image::imageops;
use mint;
use std::cmp::{max, min};
use std::io::Read;
use std::path::PathBuf;
use std::{thread, time};

trait Tile {
    fn image(&self) -> &graphics::Image;
    // eventually add actions here...
}

struct Grid {
    tiles: Vec<graphics::Image>,
    margin: usize,
    tile_width: u16,
    tile_height: u16,
    border_margin: usize,
    selected_tile: isize,
    highlight_color: graphics::Color,
    highlight_border: usize,
    tiles_per_row: usize,
    margin_to_center: usize,
}

impl Grid {
    fn new(images: Vec<graphics::Image>) -> Grid {
        let max_width = images.iter().map(|i| i.width()).fold(0, max);
        let max_height = images.iter().map(|i| i.height()).fold(0, max);
        let tile_width = min(max_width, 100);
        let tile_height = min(max_height, 100);
        Grid {
            tiles: images,
            margin: 5,
            tile_width,
            tile_height,
            border_margin: 20,
            selected_tile: 0,
            highlight_color: graphics::WHITE,
            highlight_border: 2,
            tiles_per_row: 0,
            margin_to_center: 0,
        }
    }

    fn draw_tile(&self, ctx: &mut Context, x: f32, y: f32, image: &graphics::Image) -> GameResult {
        let dest_point = mint::Point2 { x, y };
        graphics::draw(ctx, image, graphics::DrawParam::default().dest(dest_point))
    }

    fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        let mut x;
        let mut y = self.border_margin as f32;
        for (i, tile) in self.tiles.iter().enumerate() {
            x = (self.margin_to_center
                + self.border_margin
                + i % self.tiles_per_row * self.tile_width as usize
                + i % self.tiles_per_row * self.margin) as f32;
            if i != 0 && i % self.tiles_per_row == 0 {
                y += (self.margin + self.tile_height as usize) as f32;
            }
            self.draw_tile(ctx, x, y, tile)?;
            if i == self.selected_tile as usize {
                let rectangle = graphics::Mesh::new_rectangle(
                    ctx,
                    graphics::DrawMode::stroke(self.highlight_border as f32),
                    graphics::Rect::new(x, y, self.tile_width as f32, self.tile_height as f32),
                    self.highlight_color,
                )?;
                graphics::draw(ctx, &rectangle, (ggez::nalgebra::Point2::new(0.0, 0.0),))?;
            }
        }
        Ok(())
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
        println!("tiles_per_row: {}", self.tiles_per_row);
    }

    fn compute_tile_size() {}

    fn scroll_by() {}
}

struct WindowSettings {
    toggle_fullscreen: bool,
    is_fullscreen: bool,
    resize_projection: bool,
}

struct MainState {
    grid: Grid,
    zoom: f32,
    window_settings: WindowSettings,
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        //filesystem::print_all(ctx);
        let mut images = Vec::new();
        let files = filesystem::read_dir(ctx, "/")?;
        let mut count = 0;
        for file in files {
            count += 1;
            if count > 10 {
                break;
            }
            println!("{:?}", &file);
            // refactor to resize(ctx, image, max_x, max_y)
            let image = MainState::load_and_resize_image(ctx, &file)?;
            images.push(image);
        }
        Ok(MainState {
            grid: Grid::new(images),
            zoom: 1.0,
            window_settings: WindowSettings {
                toggle_fullscreen: false,
                is_fullscreen: false,
                resize_projection: true,
            },
        })
    }

    fn load_and_resize_image(ctx: &mut Context, file: &PathBuf) -> GameResult<graphics::Image> {
        let image = {
            let mut buf = Vec::new();
            let mut reader = filesystem::open(ctx, file)?;
            let _ = reader.read_to_end(&mut buf)?;
            image::load_from_memory(&buf)?.to_rgba()
        };
        let scale: f32 = 100.0 / image.width() as f32;
        let image = imageops::resize(
            &image,
            (image.width() as f32 * scale) as u32,
            (image.height() as f32 * scale) as u32,
            image::imageops::FilterType::Nearest,
        );
        let (width, height) = image.dimensions();
        graphics::Image::from_rgba8(ctx, width as u16, height as u16, &image)
    }

    fn load_image() {}
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        const DESIRED_FPS: u32 = 20;
        while timer::check_update_time(ctx, DESIRED_FPS) {
            //println!("Delta frame time: {:?} ", timer::delta(ctx));
            //println!("Average FPS: {}", timer::fps(ctx));
            //thread::sleep(time::Duration::from_millis(50));
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        /*let mut coords = graphics::screen_coordinates(ctx);
        coords.y += 1.0;
        if coords.y > 50.0 {
            coords.y = 0.0;
        }
        graphics::set_screen_coordinates(ctx, coords)?;*/
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());
        self.grid.draw(ctx)?;

        graphics::present(ctx)?;
        timer::yield_now();
        Ok(())
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymod: KeyMods) {
        println!("{} {}", self.grid.selected_tile, self.grid.tiles_per_row);
        match keycode {
            KeyCode::Up => {
                self.grid.selected_tile = max(
                    0 as isize,
                    self.grid.selected_tile - self.grid.tiles_per_row as isize,
                );
            }
            KeyCode::Down => {
                self.grid.selected_tile = min(
                    (self.grid.tiles.len() - 1) as isize,
                    self.grid.selected_tile + self.grid.tiles_per_row as isize,
                );
            }
            KeyCode::Left => {
                self.grid.selected_tile = max(0 as isize, self.grid.selected_tile - 1);
            }
            KeyCode::Right => {
                self.grid.selected_tile = min(
                    (self.grid.tiles.len() - 1) as isize,
                    self.grid.selected_tile + 1,
                );
            }
            _ => {}
        }
    }

    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) {
        println!("Resized screen to {}, {}", width, height);
        if self.window_settings.resize_projection {
            let new_rect = graphics::Rect::new(
                0.0,
                0.0,
                width as f32 * self.zoom,
                height as f32 * self.zoom,
            );
            graphics::set_screen_coordinates(ctx, new_rect).unwrap();
            self.grid.resize(new_rect.w);
        }
    }
}

fn main() -> GameResult {
    let matches = App::new("image_grid")
        .about("Utility to display images in a directory in a grid.")
        .arg(
            Arg::with_name("dir")
                .long("dir")
                .short("d")
                .takes_value(true)
                .required(true)
                .help("The directory to display in the grid."),
        )
        .get_matches();
    let cb = ggez::ContextBuilder::new("Image Grid", "Joshua Benuck")
        .add_resource_path(matches.value_of("dir").expect("Must specify a directory!"));
    let (ctx, event_loop) = &mut cb.build()?;
    let state = &mut MainState::new(ctx)?;
    graphics::set_resizable(ctx, true)?;
    event::run(ctx, event_loop, state)
}
