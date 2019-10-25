use ggez::{self, filesystem, graphics, Context, GameResult};
use image::imageops;
use regex::Regex;
use std::io::Read;
use std::path::PathBuf;

pub struct ImageLoader {
    must_not_match: Vec<String>,
    must_match: Vec<String>,
    max_count: Option<usize>,
    //images: Receiver<image::ImageBuffer>,
}

impl ImageLoader {
    pub fn new() -> ImageLoader {
        ImageLoader {
            must_not_match: Vec::new(),
            must_match: Vec::new(),
            max_count: None,
        }
    }

    pub fn filter(&mut self, filter: &str) {
        self.must_not_match.push(filter.to_owned());
    }

    pub fn only(&mut self, only: &str) {
        self.must_match.push(only.to_owned());
    }

    pub fn max(&mut self, max: usize) {
        self.max_count = Some(max);
    }

    pub fn load_all(&self, ctx: &mut Context) -> GameResult<Vec<graphics::Image>> {
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
            let image = self.load(ctx, &file);
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

    fn load_file(&self, ctx: &mut Context, file: &PathBuf) -> GameResult<image::RgbaImage> {
        let mut buf = Vec::new();
        let mut reader = filesystem::open(ctx, file)?;
        let _ = reader.read_to_end(&mut buf)?;
        Ok(image::load_from_memory(&buf)?.to_rgba())
    }

    fn load(&self, ctx: &mut Context, file: &PathBuf) -> GameResult<graphics::Image> {
        let image = self.load_file(ctx, file)?;
        let (width, height) = image.dimensions();
        graphics::Image::from_rgba8(ctx, width as u16, height as u16, &image)
    }

    fn load_and_resize(
        &self,
        ctx: &mut Context,
        file: &PathBuf,
        max_width: f32,
    ) -> GameResult<graphics::Image> {
        let image = self.load_file(ctx, file)?;
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
