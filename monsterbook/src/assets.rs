//! Embedded assets, decoded exactly once.
use crate::book::Book;
use crate::vision::Image;
use image::io::Reader;
use once_cell::sync::Lazy;
use std::io::Cursor;

fn decode_png(bytes: &[u8]) -> Image {
    Reader::new(Cursor::new(bytes))
        .with_guessed_format()
        .unwrap()
        .decode()
        .unwrap()
        .into_rgba8()
}

pub static EMPTY_CARD: Lazy<Image> =
    Lazy::new(|| decode_png(include_bytes!("assets/empty_card.png")));

pub static REFERENCE_PAGE: Lazy<Image> =
    Lazy::new(|| decode_png(include_bytes!("assets/reference_page_win.png")));

pub static BOOK: Lazy<Book> =
    Lazy::new(|| serde_json::from_str(include_str!("assets/book.json")).unwrap());

/// Seed tags for count digits 1-5, index i corresponds to count i+1.
pub static SEED_TAGS: Lazy<Vec<Image>> = Lazy::new(|| {
    [
        include_bytes!("assets/seed_tags/1.png").as_slice(),
        include_bytes!("assets/seed_tags/2.png").as_slice(),
        include_bytes!("assets/seed_tags/3.png").as_slice(),
        include_bytes!("assets/seed_tags/4.png").as_slice(),
        include_bytes!("assets/seed_tags/5.png").as_slice(),
    ]
    .iter()
    .map(|bytes| decode_png(bytes))
    .collect()
});

/// Cropped reference book pages in page order, used to identify which page a
/// screenshot belongs to. Only 22 pages exist: the final gold page has no
/// reference screenshot.
pub static REFERENCE_PAGES: Lazy<Vec<Image>> = Lazy::new(|| {
    [
        include_bytes!("assets/reference/00_red_0.png").as_slice(),
        include_bytes!("assets/reference/01_orange_0.png").as_slice(),
        include_bytes!("assets/reference/02_orange_1.png").as_slice(),
        include_bytes!("assets/reference/03_orange_2.png").as_slice(),
        include_bytes!("assets/reference/04_lightgreen_0.png").as_slice(),
        include_bytes!("assets/reference/05_lightgreen_1.png").as_slice(),
        include_bytes!("assets/reference/06_lightgreen_2.png").as_slice(),
        include_bytes!("assets/reference/07_lightgreen_3.png").as_slice(),
        include_bytes!("assets/reference/08_green_0.png").as_slice(),
        include_bytes!("assets/reference/09_green_1.png").as_slice(),
        include_bytes!("assets/reference/10_green_2.png").as_slice(),
        include_bytes!("assets/reference/11_lightblue_0.png").as_slice(),
        include_bytes!("assets/reference/12_lightblue_1.png").as_slice(),
        include_bytes!("assets/reference/13_lightblue_2.png").as_slice(),
        include_bytes!("assets/reference/14_blue_0.png").as_slice(),
        include_bytes!("assets/reference/15_blue_1.png").as_slice(),
        include_bytes!("assets/reference/16_purple_0.png").as_slice(),
        include_bytes!("assets/reference/17_purple_1.png").as_slice(),
        include_bytes!("assets/reference/18_black_0.png").as_slice(),
        include_bytes!("assets/reference/19_black_1.png").as_slice(),
        include_bytes!("assets/reference/20_gold_0.png").as_slice(),
        include_bytes!("assets/reference/21_gold_1.png").as_slice(),
    ]
    .iter()
    .map(|bytes| decode_png(bytes))
    .collect()
});
