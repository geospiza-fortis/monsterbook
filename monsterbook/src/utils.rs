use super::crop;
use super::crop::Image;
use super::stitch;
use image::{ImageError, Rgba};
use std::fs;
use std::path::Path;

pub struct PageMetadata {
    pub page_id: u8,
    pub tab_color: String,
    pub tab_index: u8,
}

pub fn page_metadata() -> Vec<PageMetadata> {
    const TAB_COUNTS: [(&str, u8); 9] = [
        ("red", 1),
        ("orange", 3),
        ("lightgreen", 4),
        ("green", 3),
        ("lightblue", 3),
        ("blue", 2),
        ("purple", 2),
        ("black", 2),
        ("gold", 3),
    ];
    let mut meta = Vec::new();
    let mut page_id = 0;
    for (color, count) in TAB_COUNTS {
        for i in 0..count {
            meta.push(PageMetadata {
                page_id: page_id,
                tab_color: color.into(),
                tab_index: i,
            });
            page_id += 1;
        }
    }
    return meta;
}

pub fn get_color(color: &str) -> Rgba<u8> {
    match color {
        "red" => Rgba([255, 102, 102, 255]),
        "orange" => Rgba([255, 187, 68, 255]),
        "lightgreen" => Rgba([221, 255, 102, 255]),
        "green" => Rgba([102, 255, 136, 255]),
        "lightblue" => Rgba([136, 255, 238, 255]),
        "blue" => Rgba([119, 187, 255, 255]),
        "purple" => Rgba([187, 119, 255, 255]),
        "black" => Rgba([85, 85, 85, 255]),
        "gold" => Rgba([255, 187, 34, 255]),
        _ => Rgba([0, 0, 0, 0]),
    }
}

pub fn get_cropped_images(source: &Path) -> Result<Vec<Image>, ImageError> {
    let mut images = Vec::new();
    // output is a file
    let (mut x, mut y) = (0, 0);
    for entry in fs::read_dir(source)? {
        let mut img = crop::imread(&entry?.path())?;
        if x == 0 && y == 0 {
            let (a, b) = crop::match_reference_page(&img)?;
            x = a;
            y = b;
        }
        let cropped = crop::crop(&mut img, x, y)?;
        images.push(cropped);
    }
    Ok(images)
}

pub fn get_empty_card_mse(images: &mut Vec<Image>) -> Vec<u32> {
    images
        .iter_mut()
        .flat_map(|img| crop::crop_cards(img).unwrap())
        .map(|img| crop::card_mse(&img))
        .collect()
}

pub fn stitch_cards(images: &Vec<Image>, width: u32) -> Image {
    // now lets crop, remove all the empty entries
    let cards = images
        .iter()
        .zip(page_metadata().into_iter())
        .flat_map(|(img, meta)| {
            crop::crop_cards(img)
                .unwrap()
                .into_iter()
                .map(|card| (card, meta.tab_color.clone()))
                .collect::<Vec<(crop::Image, String)>>()
        })
        // to determine the threshold, generate stats and look for an obvious cutoff
        .filter(|(img, _)| crop::card_mse(img) > 500)
        .map(|(mut img, color)| {
            crop::replace_background(&mut img, get_color(&color));
            img
        })
        .collect();
    println!("stitched cards");
    stitch::stitch_images(cards, width)
}
