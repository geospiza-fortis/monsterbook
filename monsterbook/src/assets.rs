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
