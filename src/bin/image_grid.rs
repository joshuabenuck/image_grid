extern crate image_grid;

use clap::{App, Arg};
use image_grid::{
    dispatcher,
    grid::{self, Grid, GridResult, TileHandler},
    image_loader::ImageLoader,
};
use opengl_graphics::Texture;

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
use piston::input::{
    keyboard::Key, Button, PressEvent, RenderArgs, RenderEvent, UpdateArgs, UpdateEvent,
};
use piston::window::WindowSettings;

pub struct ImageViewerApp {
    gl: GlGraphics, // OpenGL drawing backend.
    rotation: f64,  // Rotation for the square.
}

impl ImageViewerApp {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
        const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];

        let square = rectangle::square(0.0, 0.0, 50.0);
        let rotation = self.rotation;
        let (x, y) = (args.window_size[0] / 2.0, args.window_size[1] / 2.0);

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(GREEN, gl);

            let transform = c
                .transform
                .trans(x, y)
                .rot_rad(rotation)
                .trans(-25.0, -25.0);

            // Draw a box rotating around the middle of the screen.
            rectangle(RED, square, transform, gl);
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        // Rotate 2 radians per second.
        self.rotation += 2.0 * args.dt;
    }
}

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

    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create an Glutin window.
    let mut window: Window = WindowSettings::new("spinning-square", [200, 200])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    // Create a new game and run it.
    let mut app = ImageViewerApp {
        gl: GlGraphics::new(opengl),
        rotation: 0.0,
    };

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(r) = e.render_args() {
            app.render(&r);
        }

        if let Some(u) = e.update_args() {
            app.update(&u);
        }

        if let Some(p) = e.press_args() {
            match p {
                Button::Keyboard(key) => {
                    if key == Key::Space {
                        println!("Space!")
                    }
                }
                _ => {}
            }
        }
    }
    /*
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
        let tiles = loader.load_all(&mut ctx)?;
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
        graphics::set_resizable(&mut ctx, true)?;
        event::run(&mut ctx, &mut event_loop, &mut grid)?;
    */
    Ok(())
}
