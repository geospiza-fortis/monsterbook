use image::{imageops, io, ImageBuffer, ImageError, Rgba};
use imageproc::template_matching;
use std::path::PathBuf;
use std::io::Cursor;

type Image = ImageBuffer<Rgba<u8>, Vec<u8>>;

fn path_as_string(path: &PathBuf) -> String {
    path.clone().into_os_string().into_string().unwrap()
}

pub fn imread(source: &PathBuf) -> Result<Image, ImageError> {
    Ok(io::Reader::open(&path_as_string(source))?
        .with_guessed_format()?
        .decode()?
        .into_rgba8())
}

pub fn imsave(output: &PathBuf, img: Image) -> Result<(), ImageError> {
    img.save(&path_as_string(output))
}

fn get_reference_page() -> Result<Image, ImageError> {
    let reference_bytes = include_bytes!("assets/reference_page_win.png");
    Ok(io::Reader::new(Cursor::new(reference_bytes))
        .with_guessed_format()?
        .decode()?
        .into_rgba8())
}

fn match_reference_page(img: &Image) -> Result<(u32, u32), ImageError> {
    let reference = get_reference_page()?;
    let matched = template_matching::match_template(
        &imageops::colorops::grayscale(img),
        &imageops::colorops::grayscale(&reference),
        template_matching::MatchTemplateMethod::SumOfSquaredErrors
    );
    let extremes = template_matching::find_extremes(&matched);
    Ok(extremes.min_value_location)
}

pub fn crop(mut img: Image) -> Result<Image, ImageError> {
    let (x, y) = match_reference_page(&img)?;
    let (width, height) = (225, 165);
    Ok(imageops::crop(&mut img, x, y, width, height).to_image())
}
