use image::{imageops, io, ImageBuffer, ImageError, Rgba, RgbaImage};
use ndarray::{s, Array2};
use nshare::ToNdarray2;
use rustfft::{num_complex::Complex, FftPlanner};
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

fn into_grayscale_array(img: &Image) -> Array2<u8> {
    imageops::colorops::grayscale(img).into_ndarray2()
}

/// Take the 1d fft of each row, and then the 1d fft of each column
fn fft2d(array: &mut Array2<Complex<f32>>) {
    let mut planner = FftPlanner::<f32>::new();
    // row x col, we want to run the fft with column size
    let fft_row = planner.plan_fft_forward(array.dim().1);
    for mut row in array.rows_mut() {
        fft_row.process(row.as_slice_mut().unwrap());
    }
    let fft_col = planner.plan_fft_forward(array.dim().0);
    for mut col in array.columns_mut() {
        fft_col.process(col.slice_mut().unwrap());
    }
}

/// Reverse of fft, by inverting columns then rows
fn ifft2d(array: &mut Array2<Complex<f32>>) {
    let mut planner = FftPlanner::<f32>::new();
    // row x col, we want to run the fft with column size
    let fft_col = planner.plan_fft_inverse(array.dim().0);
    for mut col in array.columns_mut() {
        fft_col.process(col.as_slice_memory_order_mut().unwrap());
    }
    let fft_row = planner.plan_fft_inverse(array.dim().1);
    for mut row in array.rows_mut() {
        fft_row.process(row.as_slice_mut().unwrap());
    }
}

/// Run the phase correlation algorithm, and return the value into the original
/// image
fn phase_correlate(img: &mut Array2<Complex<f32>>, reference: &mut Array2<Complex<f32>>) {
    fft2d(img);
    fft2d(reference);
    // https://stackoverflow.com/a/41207820
    for (lhs, rhs) in img.iter_mut().zip(reference.iter_mut()) {
        let x = *lhs * rhs.conj();
        *lhs = x / x.norm()
    }
    ifft2d(img);
}

// pad the first image with zeros until it matches the size of the reference
fn pad_image(img: &Image, reference: &Image) -> Image {
    let mut background = RgbaImage::new(reference.width(), reference.height());
    imageops::overlay(&mut background, img, 0, 0);
    background
}

fn match_reference_page(img: &Image) -> Result<(u32, u32), ImageError> {
    // pad the reference with the original image
    let reference = pad_image(&get_reference_page()?, img);
    let mut gray_ref = into_grayscale_array(&reference).mapv(|x| Complex::new(x as f32, 0.0));
    let mut gray_img = into_grayscale_array(img).mapv(|x| Complex::new(x as f32, 0.0));
    phase_correlate(&mut gray_img, &mut gray_ref);
    // find the location of the max value
    // TODO: show the result of this matrix?
    let mut maxpos = (0, 0);
    let mut candidate = 0.0;
    for (pos, cell) in gray_img.indexed_iter() {
        let normed = cell.norm();
        if normed > candidate {
            candidate = normed;
            maxpos = pos;
        }
    }
    Ok((maxpos.0 as u32, maxpos.1 as u32))
}

pub fn crop(mut img: Image) -> Result<Image, ImageError> {
    let (x, y) = match_reference_page(&img)?;
    let (width, height) = (225, 165);
    Ok(imageops::crop(&mut img, x, y, width, height).to_image())
}
