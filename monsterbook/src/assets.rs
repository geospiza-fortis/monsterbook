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

/// The empty-card template: the per-pixel mean of all 650 card slots across
/// the 26 empty-book pages in `data/processed/empty_new/` (see its
/// REPORT.md), at native WIN_HD card scale (33x45). Replaces the original
/// `empty_card.png` (still on disk, no longer compiled in), against which
/// empty cards from the 800x600-client captures scored up to 2924 — well
/// past the old 500 threshold.
pub static EMPTY_CARD: Lazy<Image> =
    Lazy::new(|| decode_png(include_bytes!("assets/empty_card_v2.png")));

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

/// The 26 embedded reference pages, one per `book.json` page in page order,
/// at native WIN_HD (165x225) scale: direct crops of an *empty* book
/// (`data/processed/empty_new/{00..25}.png`, see its REPORT.md), not
/// resized from a higher-resolution source, so no rescale step is needed.
///
/// Empty pages replace the `reference_v2` crops of a collected book
/// (still on disk, no longer compiled in): page identity comes from the
/// stable page chrome (tab ribbon, page label) rather than from
/// user-specific card sprites, and this matters in practice. The collected
/// set identified the 26 empty pages only 25/26 (empty page 03 matched
/// reference 21, whose "populated" crop happened to be empty), while the
/// empty set identifies every page of the fully collected book 26/26 —
/// the worst-case card-content mismatch.
pub static REFERENCE_PAGES_WIN: Lazy<Vec<Image>> = Lazy::new(|| {
    [
        include_bytes!("assets/reference_v3/00.png").as_slice(),
        include_bytes!("assets/reference_v3/01.png").as_slice(),
        include_bytes!("assets/reference_v3/02.png").as_slice(),
        include_bytes!("assets/reference_v3/03.png").as_slice(),
        include_bytes!("assets/reference_v3/04.png").as_slice(),
        include_bytes!("assets/reference_v3/05.png").as_slice(),
        include_bytes!("assets/reference_v3/06.png").as_slice(),
        include_bytes!("assets/reference_v3/07.png").as_slice(),
        include_bytes!("assets/reference_v3/08.png").as_slice(),
        include_bytes!("assets/reference_v3/09.png").as_slice(),
        include_bytes!("assets/reference_v3/10.png").as_slice(),
        include_bytes!("assets/reference_v3/11.png").as_slice(),
        include_bytes!("assets/reference_v3/12.png").as_slice(),
        include_bytes!("assets/reference_v3/13.png").as_slice(),
        include_bytes!("assets/reference_v3/14.png").as_slice(),
        include_bytes!("assets/reference_v3/15.png").as_slice(),
        include_bytes!("assets/reference_v3/16.png").as_slice(),
        include_bytes!("assets/reference_v3/17.png").as_slice(),
        include_bytes!("assets/reference_v3/18.png").as_slice(),
        include_bytes!("assets/reference_v3/19.png").as_slice(),
        include_bytes!("assets/reference_v3/20.png").as_slice(),
        include_bytes!("assets/reference_v3/21.png").as_slice(),
        include_bytes!("assets/reference_v3/22.png").as_slice(),
        include_bytes!("assets/reference_v3/23.png").as_slice(),
        include_bytes!("assets/reference_v3/24.png").as_slice(),
        include_bytes!("assets/reference_v3/25.png").as_slice(),
    ]
    .iter()
    .map(|bytes| decode_png(bytes))
    .collect()
});
