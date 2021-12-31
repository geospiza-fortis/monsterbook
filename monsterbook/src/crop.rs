use image::{imageops, io, ImageBuffer, ImageError, Rgba};
use std::path::PathBuf;

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

pub fn crop(mut img: Image) -> Image {
    let (x, y) = (152, 295);
    let (width, height) = (225, 165);
    imageops::crop(&mut img, x, y, width, height).to_image()
}
