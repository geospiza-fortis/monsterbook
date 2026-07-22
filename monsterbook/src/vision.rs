//! Pure image primitives: no file I/O, no embedded assets.
use crate::layout::Layout;
use image::{imageops, ImageBuffer, Rgba, RgbaImage};
use ndarray::{stack, Array2, ArrayBase, Axis, ViewRepr};
use nshare::ToNdarray2;
use rustfft::{num_complex::Complex, FftDirection, FftPlanner};

pub type Image = ImageBuffer<Rgba<u8>, Vec<u8>>;

pub fn into_grayscale_array(img: &Image) -> Array2<u8> {
    imageops::colorops::grayscale(img).into_ndarray2()
}

fn column_fft(
    array: &mut Array2<Complex<f32>>,
    planner: &mut FftPlanner<f32>,
    direction: FftDirection,
) {
    // gross, since we end up creating a new array that's transposed...
    let plan = match direction {
        FftDirection::Forward => planner.plan_fft_forward(array.dim().0),
        FftDirection::Inverse => planner.plan_fft_inverse(array.dim().0),
    };
    let mut cols = Vec::new();
    for j in 0..array.dim().1 {
        let mut selected = array.t().select(Axis(0), &[j]);
        plan.process(selected.row_mut(0).as_slice_mut().unwrap());
        cols.push(selected);
    }
    let stacked: Vec<ArrayBase<ViewRepr<&_>, _>> = cols.iter().map(|r| r.row(0)).collect();
    *array = stack(Axis(0), stacked.as_slice()).unwrap();
}

/// Take the 1d fft of each row, and then the 1d fft of each column
fn fft2d(array: &mut Array2<Complex<f32>>) {
    let mut planner = FftPlanner::<f32>::new();
    // row x col, we want to run the fft with column size
    let fft_row = planner.plan_fft_forward(array.dim().1);
    for mut row in array.rows_mut() {
        fft_row.process(row.as_slice_mut().unwrap());
    }
    column_fft(array, &mut planner, FftDirection::Forward);
}

/// Reverse of fft, by inverting columns then rows
fn ifft2d(array: &mut Array2<Complex<f32>>) {
    // we make an assumption that the data is already in column-major form due
    // to the use of fft2d.
    let mut planner = FftPlanner::<f32>::new();
    let fft_row = planner.plan_fft_inverse(array.dim().1);
    for mut row in array.rows_mut() {
        fft_row.process(row.as_slice_mut().unwrap());
    }
    column_fft(array, &mut planner, FftDirection::Inverse);
}

/// Run the phase correlation algorithm, and return the value into the original
/// image: https://stackoverflow.com/a/32664730
fn phase_correlate(img: &mut Array2<Complex<f32>>, reference: &mut Array2<Complex<f32>>) {
    fft2d(img);
    fft2d(reference);
    // https://stackoverflow.com/a/41207820
    for (lhs, rhs) in img.iter_mut().zip(reference.iter_mut()) {
        let x = *lhs * rhs.conj();
        *lhs = x / x.norm();
    }
    ifft2d(img);
}

// pad the first image with zeros until it matches the size of the reference
pub fn pad_image(img: &Image, reference: &Image) -> Image {
    let mut background = RgbaImage::new(reference.width(), reference.height());
    imageops::overlay(&mut background, img, 0, 0);
    background
}

/// Find the (x, y) offset of the reference page within the image via phase
/// correlation.
pub fn match_reference_page(img: &Image, reference_page: &Image) -> (u32, u32) {
    // pad the reference with the original image
    let reference = pad_image(reference_page, img);
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
    (maxpos.1 as u32, maxpos.0 as u32)
}

pub fn crop_page(img: &mut Image, x: u32, y: u32, layout: &Layout) -> Image {
    imageops::crop(img, x, y, layout.page_width, layout.page_height).to_image()
}

pub fn crop_cards(img: &Image, layout: &Layout) -> Vec<Image> {
    let num_rows = layout.grid_rows;
    let num_cols = layout.grid_cols;
    let h = img.height() / num_rows;
    let w = img.width() / num_cols;
    let mut page = img.clone();
    let mut cards = Vec::new();
    for i in 0..num_rows {
        for j in 0..num_cols {
            let card = imageops::crop(&mut page, j * w, i * h, w, h).to_image();
            cards.push(card);
        }
    }
    cards
}

/// Crop the count tag (the "x<n>" badge in the lower left) from a card.
pub fn crop_tag(card: &Image, layout: &Layout) -> Image {
    let mut cloned = card.clone();
    imageops::crop(
        &mut cloned,
        layout.tag_x,
        layout.tag_y,
        layout.tag_width,
        layout.tag_height,
    )
    .to_image()
}

/// Sobel edge-magnitude of a grayscale image, computed the same way as the
/// old JS app's `sobel()` in `src/image.js`: convolve with the horizontal and
/// vertical 3x3 Sobel kernels and sum the (unsigned) results per-pixel.
/// Operating on edge magnitude rather than raw grayscale is far more robust
/// to the small (5-10%) crop-margin drift seen on real screenshots, since
/// flat page-background regions (which dominate raw MSE and are nearly
/// identical across all pages) contribute ~0 while card borders and art
/// dominate the signal.
pub fn sobel_magnitude(img: &Image) -> Array2<i32> {
    let gray = into_grayscale_array(img).mapv(|x| x as i32);
    let (h, w) = gray.dim();
    let gx_k: [[i32; 3]; 3] = [[1, 0, -1], [2, 0, -2], [1, 0, -1]];
    let gy_k: [[i32; 3]; 3] = [[1, 2, 1], [0, 0, 0], [-1, -2, -1]];
    let mut out = Array2::<i32>::zeros((h, w));
    for y in 0..h {
        for x in 0..w {
            let mut gx = 0i32;
            let mut gy = 0i32;
            for dy in 0..3usize {
                for dx in 0..3usize {
                    // clamp-to-edge sampling at the border
                    let sy = (y as isize + dy as isize - 1).clamp(0, h as isize - 1) as usize;
                    let sx = (x as isize + dx as isize - 1).clamp(0, w as isize - 1) as usize;
                    let v = gray[[sy, sx]];
                    gx += v * gx_k[dy][dx];
                    gy += v * gy_k[dy][dx];
                }
            }
            out[[y, x]] = gx.abs() + gy.abs();
        }
    }
    out
}

/// Mean-squared error between the Sobel edge magnitudes of two images. See
/// `sobel_magnitude` for why this is more robust to crop drift than raw
/// grayscale `mse`.
pub fn mse_edges(img: &Image, reference: &Image) -> u32 {
    let a = sobel_magnitude(img);
    let b = sobel_magnitude(reference);
    let acc: i64 = a
        .iter()
        .zip(b.iter())
        .map(|(x, y)| ((*x - *y) as i64).pow(2))
        .sum();
    let denom = (img.width() * img.height()) as i64;
    (acc / denom) as u32
}

/// Zero-mean normalized cross-correlation between two same-size grayscale
/// images, in [-1, 1] (1 = identical up to a linear brightness/contrast
/// shift). Unlike MSE this is invariant to the small global
/// brightness/contrast differences between screenshots (different
/// monitor/gamma settings, JPEG-vs-PNG capture, etc.), which is a plausible
/// source of the poor MSE margins seen on real screenshots.
pub fn ncc(img: &Image, reference: &Image) -> f64 {
    let a = into_grayscale_array(img).mapv(|x| x as f64);
    let b = into_grayscale_array(reference).mapv(|x| x as f64);
    let mean_a = a.mean().unwrap();
    let mean_b = b.mean().unwrap();
    let mut num = 0.0f64;
    let mut da = 0.0f64;
    let mut db = 0.0f64;
    for (x, y) in a.iter().zip(b.iter()) {
        let dx = x - mean_a;
        let dy = y - mean_b;
        num += dx * dy;
        da += dx * dx;
        db += dy * dy;
    }
    if da == 0.0 || db == 0.0 {
        return 0.0;
    }
    num / (da.sqrt() * db.sqrt())
}

pub fn mse(img: &Image, reference: &Image) -> u32 {
    let gray_img = into_grayscale_array(img);
    let gray_ref = into_grayscale_array(reference);

    // calculatinng mse
    let acc: i32 = gray_img
        .iter()
        .zip(gray_ref.iter())
        .map(|(x, y)| (*x as i32 - *y as i32).pow(2))
        .sum();
    let denom = img.width() * img.height();
    return (acc / denom as i32) as u32;
}

// remove the background from a card
pub fn replace_background(img: &mut Image, color: Rgba<u8>, layout: &Layout) {
    // replace the background with our own custom color
    let mut background = RgbaImage::from_fn(img.width(), img.height(), |_, _| color);
    let mut cloned = img.clone();
    let cropped = imageops::crop(
        &mut cloned,
        layout.content_x,
        layout.content_y,
        layout.content_width,
        layout.content_height,
    );
    imageops::overlay(&mut background, &cropped, layout.content_x, layout.content_y);
    *img = background;
}

pub fn stitch_images(images: Vec<Image>, width: u32) -> Image {
    if images.is_empty() {
        return Image::new(0, 0);
    }
    let x = images[0].width();
    let y = images[0].height();
    let n = images.len();
    let height = (n as f32 / width as f32).ceil() as u32;
    let mut background = RgbaImage::new(x * width, y * height);
    for i in 0..height {
        for j in 0..width {
            let index = (i * width + j) as usize;
            if index >= n {
                break;
            }
            let img = &images[index];
            imageops::overlay(&mut background, img, j * x, i * y);
        }
    }
    background
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stitch_images_dimensions() {
        let images: Vec<Image> = (0..5).map(|_| RgbaImage::new(10, 20)).collect();
        let stitched = stitch_images(images, 2);
        // 5 images, 2 per row -> 3 rows
        assert_eq!(stitched.width(), 20);
        assert_eq!(stitched.height(), 60);
    }

    #[test]
    fn test_crop_tag_geometry() {
        use crate::layout::WIN_HD;
        // a full-size card is 33x45; the tag is the 6x9 rect at (5, 31)
        let mut card = RgbaImage::new(33, 45);
        for y in 31..40 {
            for x in 5..11 {
                card.put_pixel(x, y, Rgba([255, 255, 255, 255]));
            }
        }
        let tag = crop_tag(&card, &WIN_HD);
        assert_eq!(tag.width(), 6);
        assert_eq!(tag.height(), 9);
        // every pixel of the crop comes from the marked region
        assert!(tag.pixels().all(|p| *p == Rgba([255, 255, 255, 255])));
    }

    #[test]
    fn test_mse_identical_is_zero() {
        let img = RgbaImage::from_pixel(4, 4, Rgba([100, 150, 200, 255]));
        assert_eq!(mse(&img, &img.clone()), 0);
    }
}
