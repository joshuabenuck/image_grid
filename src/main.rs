use clap::{App, Arg};
use ggez::event::{self, KeyCode, KeyMods, MouseButton};
use ggez::{self, filesystem, graphics, timer, Context, GameResult};
use image::imageops;
use mint;
use regex::Regex;
use std::cmp::{max, min};
use std::io::Read;
use std::path::PathBuf;

trait Tile {
    fn image(&self) -> &graphics::Image;
    // eventually add actions here...
}

// margin: the total space between items the grid
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
    coords_to_select: Option<(f32, f32)>,
}

impl Grid {
    fn new(images: Vec<graphics::Image>) -> Grid {
        let max_width = images.iter().map(|i| i.width()).fold(0, max);
        let max_height = images.iter().map(|i| i.height()).fold(0, max);
        let tile_width = min(max_width, 200);
        let tile_height = min(max_height, 200);
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
            coords_to_select: None,
        }
    }

    fn margin<'a>(&'a mut self, m: usize) -> &'a mut Self {
        self.margin = m;
        self
    }

    fn draw_tile(&self, ctx: &mut Context, x: f32, y: f32, image: &graphics::Image) -> GameResult {
        let dest_point = mint::Point2 { x, y };
        graphics::draw(ctx, image, graphics::DrawParam::default().dest(dest_point))
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<f32> {
        let mut x;
        let mut y = self.border_margin as f32;
        let mut scroll_pos = 0.0;
        for (i, tile) in self.tiles.iter().enumerate() {
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
            self.draw_tile(ctx, x, y, tile)?;
            if i == self.selected_tile as usize {
                let rectangle = graphics::Mesh::new_rectangle(
                    ctx,
                    graphics::DrawMode::stroke(self.highlight_border as f32),
                    graphics::Rect::new(x, y, self.tile_width as f32, self.tile_height as f32),
                    self.highlight_color,
                )?;
                graphics::draw(ctx, &rectangle, (ggez::nalgebra::Point2::new(0.0, 0.0),))?;
                let mut screen = graphics::screen_coordinates(ctx);
                if y + self.tile_height as f32 > screen.y + screen.h {
                    screen.y += self.tile_height as f32;
                    graphics::set_screen_coordinates(ctx, screen)?;
                    scroll_pos = screen.y;
                }
                if y < screen.y {
                    screen.y -= self.tile_height as f32;
                    graphics::set_screen_coordinates(ctx, screen)?;
                    scroll_pos = screen.y;
                }
            }
        }
        Ok(scroll_pos)
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

struct MainState {
    grid: Grid,
    scroll_pos: f32,
}

impl MainState {
    fn new(grid: Grid) -> GameResult<MainState> {
        //filesystem::print_all(ctx);
        Ok(MainState {
            grid,
            scroll_pos: 0.0,
        })
    }
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
        self.scroll_pos = self.grid.draw(ctx)?;

        graphics::present(ctx)?;
        timer::yield_now();
        Ok(())
    }

    fn mouse_button_up_event(&mut self, ctx: &mut Context, _button: MouseButton, x: f32, y: f32) {
        let screen = graphics::screen_coordinates(ctx);
        self.grid.select_tile_under(x, y + screen.y);
    }

    fn mouse_wheel_event(&mut self, _ctx: &mut Context, _x: f32, y: f32) {
        if y > 0.0 {
            self.grid.up();
        }
        if y < 0.0 {
            self.grid.down();
        }
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymod: KeyMods) {
        println!("{} {}", self.grid.selected_tile, self.grid.tiles_per_row);
        match keycode {
            KeyCode::Up => {
                self.grid.up();
            }
            KeyCode::Down => {
                self.grid.down();
            }
            KeyCode::Left => {
                self.grid.left();
            }
            KeyCode::Right => {
                self.grid.right();
            }
            _ => {}
        }
    }

    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) {
        println!("Resized screen to {}, {}", width, height);
        let mut screen = graphics::screen_coordinates(ctx);
        screen.w = width as f32;
        screen.h = height as f32;
        graphics::set_screen_coordinates(ctx, screen).unwrap();
        self.grid.resize(screen.w);
    }
}

struct ImageLoader {
    must_not_match: Vec<String>,
    must_match: Vec<String>,
    max_count: Option<usize>,
    //images: Receiver<image::ImageBuffer>,
}

impl ImageLoader {
    fn new() -> ImageLoader {
        ImageLoader {
            must_not_match: Vec::new(),
            must_match: Vec::new(),
            max_count: None,
        }
    }

    fn filter(&mut self, filter: &str) {
        self.must_not_match.push(filter.to_owned());
    }

    fn only(&mut self, only: &str) {
        self.must_match.push(only.to_owned());
    }

    fn max(&mut self, max: usize) {
        self.max_count = Some(max);
    }

    fn load_all(&self, ctx: &mut Context) -> GameResult<Vec<graphics::Image>> {
        let mut images = Vec::new();
        let files = filesystem::read_dir(ctx, "/")?;
        let mut count = 0;
        let must_not_match: Vec<Regex> = self
            .must_not_match
            .iter()
            .map(|f| Regex::new(f).expect(format!("Regex error for 'filter': {}", f).as_str()))
            .collect();
        let must_match: Vec<Regex> = self
            .must_match
            .iter()
            .map(|f| Regex::new(f).expect(format!("Regex error for 'only': {}", f).as_str()))
            .collect();
        'fileloop: for file in files {
            // Is there a way to do this more concisely?
            if let Some(max) = self.max_count {
                if count >= max {
                    break;
                }
            }
            //println!("{:?}", &file);
            // refactor to resize(ctx, image, max_x, max_y)
            if file.is_dir() {
                continue;
            }
            let filestr = file
                .to_str()
                .expect("Unable to convert image filename to str");
            for regex in &must_match {
                if !regex.is_match(&filestr) {
                    continue 'fileloop;
                }
            }
            for regex in &must_not_match {
                println!("{}, {:?}", &filestr, regex);
                if regex.is_match(&filestr) {
                    continue 'fileloop;
                }
            }
            let image = self.load_and_resize(ctx, &file, 200.0);
            match image {
                Ok(i) => {
                    count += 1;
                    images.push(i);
                }
                Err(err) => eprintln!("{}: {}", file.display(), err),
            }
        }
        Ok(images)
    }

    fn load_and_resize(
        &self,
        ctx: &mut Context,
        file: &PathBuf,
        max_width: f32,
    ) -> GameResult<graphics::Image> {
        let image = {
            let mut buf = Vec::new();
            let mut reader = filesystem::open(ctx, file)?;
            let _ = reader.read_to_end(&mut buf)?;
            image::load_from_memory(&buf)?.to_rgba()
        };
        let scale: f32 = max_width / image.width() as f32;
        let image = imageops::resize(
            &image,
            (image.width() as f32 * scale) as u32,
            (image.height() as f32 * scale) as u32,
            image::imageops::FilterType::Nearest,
        );
        let (width, height) = image.dimensions();
        graphics::Image::from_rgba8(ctx, width as u16, height as u16, &image)
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
    let grid = Grid::new(loader.load_all(&mut ctx)?);
    let state = &mut MainState::new(grid)?;
    graphics::set_resizable(&mut ctx, true)?;
    event::run(&mut ctx, &mut event_loop, state)
}
