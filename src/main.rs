mod image_loader;
use clap::{App, Arg};
use ggez::event::{self, KeyCode, KeyMods, MouseButton};
use ggez::{self, graphics, timer, Context, GameResult};
use image_loader::ImageLoader;
use mint;
use std::cmp::{max, min};

enum NextAction {
    None,
    Push(Box<dyn Widget>),
    Pop,
    Replace(Box<dyn Widget>),
}

trait Widget: event::EventHandler {
    fn next(&self) -> NextAction;
}

trait Tile {
    fn image(&self) -> &graphics::Image;
}

struct ImageViewer {
    image: graphics::Image,
}

impl Tile for ImageViewer {
    fn image(&self) -> &graphics::Image {
        &self.image
    }
}

impl Widget for ImageViewer {
    fn next(&self) -> NextAction {
        NextAction::None
    }
}

impl event::EventHandler for ImageViewer {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }
    fn draw(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }
}

// margin: the total space between items the grid
struct Grid {
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
}

impl Grid {
    fn new(tiles: Vec<Box<dyn Tile>>) -> Grid {
        let images: Vec<&graphics::Image> = tiles.iter().map(|t| t.image()).collect();
        // TODO: Fix by precomputing all image sizes.
        let max_width = (&images).iter().map(|i| i.width()).fold(0, max);
        let max_height = (&images).iter().map(|i| i.height()).fold(0, max);
        let tile_width = min(max_width, 200);
        let tile_height = min(max_height, 200);
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
        }
    }

    fn margin<'a>(&'a mut self, m: usize) -> &'a mut Self {
        self.margin = m;
        self
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
            //thread::sleep(time::Duration::from_millis(50));
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());
        let mut x;
        let mut y = self.border_margin as f32;
        let mut screen = graphics::screen_coordinates(ctx);
        for (i, tile) in self.tiles.iter().enumerate() {
            let image = tile.image();
            let scale = f32::min(
                self.tile_width as f32 / image.width() as f32,
                self.tile_height as f32 / image.height() as f32,
            );
            let width = image.width() as f32 * scale;
            let height = image.height() as f32 * scale;
            x = (self.margin_to_center
                + self.border_margin
                + i % self.tiles_per_row * self.tile_width as usize
                + i % self.tiles_per_row * self.margin) as f32;
            if i != 0 && i % self.tiles_per_row == 0 {
                y += (self.margin + self.tile_height as usize) as f32;
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
                    screen.y += height;
                    graphics::set_screen_coordinates(ctx, screen)?;
                }
                if y < screen.y {
                    screen.y -= height;
                    graphics::set_screen_coordinates(ctx, screen)?;
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
            self.up();
        }
        if y < 0.0 {
            self.down();
        }
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymod: KeyMods) {
        println!("{} {}", self.selected_tile, self.tiles_per_row);
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
                self.draw_tile = !self.draw_tile;
            }
            KeyCode::Escape => {}
            _ => {}
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

struct Dispatcher {
    widget: Box<dyn Widget>,
    parent: Option<Box<dyn Widget>>,
}

impl event::EventHandler for Dispatcher {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let value = (*self.widget).update(ctx);
        match (*self.widget).next() {
            NextAction::None => (),
            NextAction::Push(widget) => {
                self.parent = Some(std::mem::replace(&mut self.widget, widget));
            }
            NextAction::Pop => {
                //std::mem::replace(&mut self.parent, ...)
            }
            NextAction::Replace(widget) => self.widget = widget,
        }
        value
    }
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        (*self.widget).draw(ctx)
    }

    fn mouse_button_up_event(&mut self, ctx: &mut Context, _button: MouseButton, x: f32, y: f32) {
        (*self.widget).mouse_button_up_event(ctx, _button, x, y)
    }

    fn mouse_wheel_event(&mut self, _ctx: &mut Context, _x: f32, y: f32) {
        (*self.widget).mouse_wheel_event(_ctx, _x, y)
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymod: KeyMods) {
        (*self.widget).key_up_event(_ctx, keycode, _keymod)
    }

    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) {
        (*self.widget).resize_event(ctx, width, height)
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
                .help("The directory to display."),
        )
        .arg(
            Arg::with_name("max")
                .long("max")
                .short("m")
                .takes_value(true)
                .required(true)
                .default_value("100")
                .help("The maximum number of images to display."),
        )
        .arg(
            Arg::with_name("filter")
                .long("filter")
                .short("f")
                .multiple(true)
                .takes_value(true)
                .help("Filter out files that match the regex."),
        )
        .arg(
            Arg::with_name("only")
                .long("only")
                .short("o")
                .multiple(true)
                .takes_value(true)
                .help("Only display files that match this regex."),
        )
        .get_matches();
    let cb = ggez::ContextBuilder::new("Image Grid", "Joshua Benuck")
        .add_resource_path(matches.value_of("dir").expect("Must specify a directory!"));
    let (mut ctx, mut event_loop) = cb.build()?;
    let mut loader = ImageLoader::new();
    if let Some(filters) = matches.values_of("filter") {
        for filter in filters {
            loader.filter(filter);
        }
    }
    if let Some(onlys) = matches.values_of("only") {
        for only in onlys {
            loader.only(only);
        }
    }
    if let Some(max) = matches.value_of("max") {
        let max = max.parse().expect("Unable to parse max");
        loader.max(max);
    }
    let grid = Grid::new(
        loader
            .load_all(&mut ctx)?
            .into_iter()
            .map(|i| Box::new(ImageViewer { image: i }) as Box<dyn Tile>)
            .collect(),
    );
    graphics::set_resizable(&mut ctx, true)?;
    event::run(
        &mut ctx,
        &mut event_loop,
        &mut Dispatcher {
            widget: Box::new(grid),
            parent: None,
        },
    )?;
    Ok(())
}
