use convolve2d::{convolve2d, SubPixels, DynamicMatrix};
use image::{imageops, io, GrayImage, ImageBuffer, ImageError, Rgba};
use imageproc::template_matching;
use std::io::Cursor;
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

fn get_reference_page() -> Result<Image, ImageError> {
    let reference_bytes = include_bytes!("assets/search_icon.png");
    Ok(io::Reader::new(Cursor::new(reference_bytes))
        .with_guessed_format()?
        .decode()?
        .into_rgba8())
}

fn match_reference_page(img: &Image) -> Result<(u32, u32), ImageError> {
    let reference = get_reference_page()?;
    let gray_ref: DynamicMatrix<SubPixels<u8, 1>> = imageops::colorops::grayscale(&reference).into();
    let gray_img: DynamicMatrix<SubPixels<u8, 1>> = imageops::colorops::grayscale(img).into();
    
    let convolution = convolve2d(
        &gray_img.map(|x| x.0[0] as i32),
        &gray_ref.map(|x| x.0[0] as i32),
    );
    //let buffer: ImageBuffer<Luma<f32>, Vec<f32>> = convolution.into();
    let gray = GrayImage::from(convolution.map(|x| SubPixels([x.abs() as u8])));
    let extremes = template_matching::find_extremes(&gray);
    Ok(extremes.min_value_location)
}

pub fn crop(mut img: Image) -> Result<Image, ImageError> {
    let (x, y) = match_reference_page(&img)?;
    let (width, height) = (225, 165);
    Ok(imageops::crop(&mut img, x, y, width, height).to_image())
}
