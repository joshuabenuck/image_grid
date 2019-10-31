extern crate image_grid;

use clap::{App, Arg};
use dispatcher::Dispatcher;
use ggez::event;
use ggez::{self, graphics, GameResult};
use grid::{Grid, TileHandler};
use image_grid::{dispatcher, grid, image_loader::ImageLoader};

trait Game {}

struct ImageTileHandler {
    tiles: Vec<graphics::Image>,
}

impl TileHandler for ImageTileHandler {
    fn tiles(&self) -> &Vec<graphics::Image> {
        return &self.tiles;
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
    let mut handler = ImageTileHandler {
        tiles: loader.load_all(&mut ctx)?,
    };

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
    graphics::set_resizable(&mut ctx, true)?;
    event::run(&mut ctx, &mut event_loop, &mut grid)?;
    Ok(())
}
