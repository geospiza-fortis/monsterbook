use image::{imageops, ImageBuffer, Rgba, RgbaImage};

type Image = ImageBuffer<Rgba<u8>, Vec<u8>>;

pub fn stitch_images(images: Vec<Image>, width: u32, height: u32) -> Image {
    let x = images[0].width();
    let y = images[0].height();
    let mut background = RgbaImage::new(x * width, y * height);
    for i in 0..height {
        for j in 0..width {
            let index = (i * width + j) as usize;
            if index >= images.len() {
                break;
            }
            let img = &images[index];
            imageops::overlay(&mut background, img, j * x, i * y);
        }
    }
    background
}
