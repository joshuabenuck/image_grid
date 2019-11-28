extern crate image_grid;

use clap::{App, Arg};
use env_logger;
use glutin_window::GlutinWindow as Window;
use image_grid::{
    grid::{run, GridController, GridModel, GridResult, GridView, TileController},
    image_loader::ImageLoader,
};
use opengl_graphics::Texture;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::window::WindowSettings;
use std::io::{self, BufRead};
use std::path::PathBuf;

struct ImageTile {}

impl TileController for ImageTile {}

struct Images {
    filenames: Vec<String>,
    tiles: Vec<Texture>,
    indexes: Vec<usize>,
}

impl GridModel for Images {
    fn tiles(&self) -> &Vec<usize> {
        return &self.indexes;
    }

    fn tile(&self, i: usize) -> &Texture {
        &self.tiles[i]
    }
}

fn main() -> GridResult<()> {
    env_logger::init();
    let matches = App::new("image_grid")
        .about("Utility to display images in a directory in a grid.")
        .arg(
            Arg::with_name("dir")
                .long("dir")
                .short("d")
                .takes_value(true)
                .help("The directory to display."),
        )
        .arg(
            Arg::with_name("stdin")
                .long("stdin")
                .help("Read files to display from stdin"),
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
        .arg(
            Arg::with_name("draw-tile")
                .long("draw-tile")
                .takes_value(true)
                .default_value("true")
                .help("Whether to draw tile fullscreen when activating a tile."),
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

    let (files, tiles) = if matches.is_present("dir") {
        loader.load_all(PathBuf::from(
            matches.value_of("dir").expect("Must specify a directory!"),
        ))?
    } else if matches.is_present("stdin") {
        let mut files: Vec<PathBuf> = Vec::new();
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let file = PathBuf::from(line?);
            if !file.exists() {
                eprintln!("Skipping: {}", &file.display());
                continue;
            }
            files.push(file);
        }
        loader.load_files(files)?
    } else {
        panic!("Must specify either --dir or --stdin. See --help for details.");
    };
    let indexes: Vec<usize> = (0..tiles.len()).collect();
    let model: Box<dyn GridModel> = Box::new(Images {
        filenames: files,
        tiles,
        indexes,
    });
    let mut controller = GridController::new(model, ImageTile {});

    let mut view = GridView::new(
        matches
            .value_of("tile-width")
            .unwrap()
            .parse::<usize>()
            .unwrap(),
        matches
            .value_of("tile-height")
            .unwrap()
            .parse::<usize>()
            .unwrap(),
    );
    let draw_tile = matches.value_of("draw-tile").unwrap().parse::<bool>()?;
    if !draw_tile {
        controller.allow_draw_tile = false;
    }
    run(&mut window, &mut gl, &mut controller, &mut view)?;
    Ok(())
}
