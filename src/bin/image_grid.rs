extern crate image_grid;

use clap::{App, Arg};
use image_grid::{
    dispatcher,
    grid::{self, EventHandler, Grid, GridResult, TileHandler},
    image_loader::ImageLoader,
};
use opengl_graphics::Texture;
use std::path::PathBuf;
use std::thread;
use std::time;

trait Game {}

struct ImageTileHandler {
    tiles: Vec<Texture>,
    indexes: Vec<usize>,
}

impl TileHandler for ImageTileHandler {
    fn tiles(&self) -> &Vec<usize> {
        return &self.indexes;
    }

    fn tile(&self, i: usize) -> &Texture {
        &self.tiles[i]
    }
}

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::*;
use piston::input::{keyboard::ModifierKey, Button, PressEvent, RenderEvent, UpdateEvent};
use piston::window::WindowSettings;

fn main() -> GridResult<()> {
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
        .arg(
            Arg::with_name("tile-width")
                .long("tile-width")
                .takes_value(true)
                .default_value("200")
                .help("Set the max tile-width."),
        )
        .arg(
            Arg::with_name("tile-height")
                .long("tile-width")
                .takes_value(true)
                .default_value("200")
                .help("Set the max tile-width."),
        )
        .get_matches();

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
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create an Glutin window.
    let mut window: Window = WindowSettings::new("Doorways", [800, 600])
        .resizable(true)
        .vsync(true)
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut gl = GlGraphics::new(opengl);
    //let mut app = ImageViewerApp { gl, rotation: 0.0 };

    let tiles = loader.load_all(PathBuf::from(
        matches.value_of("dir").expect("Must specify a directory!"),
    ))?;
    let indexes = (0..tiles.len()).collect();
    let mut handler = ImageTileHandler { tiles, indexes };
    let mut grid = Grid::new(
        Box::new(&mut handler),
        matches
            .value_of("tile-width")
            .unwrap()
            .parse::<u16>()
            .unwrap(),
        matches
            .value_of("tile-height")
            .unwrap()
            .parse::<u16>()
            .unwrap(),
    );

    let mut settings = EventSettings::new();
    settings.set_lazy(true);
    settings.swap_buffers(true);
    settings.max_fps(1);
    settings.ups(1);
    let mut events = Events::new(settings);
    while let Some(e) = events.next(&mut window) {
        if let Some(r) = e.render_args() {
            grid.draw(&mut gl, &r)?;
        }

        if let Some(_u) = e.update_args() {
            grid.update(&mut gl)?;
        }

        if let Some(p) = e.press_args() {
            match p {
                Button::Keyboard(key) => {
                    grid.key_down_event(key, ModifierKey::NO_MODIFIER, false);
                }
                _ => {}
            }
        }
    }
    /*

        graphics::set_resizable(&mut ctx, true)?;
        event::run(&mut ctx, &mut event_loop, &mut grid)?;
    */
    Ok(())
}
