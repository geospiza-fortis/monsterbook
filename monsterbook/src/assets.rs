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

/// The 26 embedded reference pages, one per `book.json` page in page order,
/// at native WIN_HD (165x225) scale: they are direct crops of real
/// screenshots (`data/processed/reference_new/{00..25}.png`, see
/// `data/processed/reference_new/REPORT.md`), not resized from a
/// higher-resolution source, so no rescale step is needed.
///
/// This replaces the pre-Phase-5 mac-scale `REFERENCE_PAGES` (22 images,
/// downscaled 2x at load time to line up with WIN_HD) which is now dropped
/// entirely: it only covered 22 of the book's (now) 26 pages, its
/// page-order no longer matches the new `book.json`, and dropping it saves
/// ~1.6MB of embedded (and wasm-shipped) image data that would otherwise be
/// redundant with these 26 native-scale crops. The old mac reference PNGs
/// remain on disk under `src/assets/reference/` (and in git history) but
/// are no longer compiled in.
pub static REFERENCE_PAGES_WIN: Lazy<Vec<Image>> = Lazy::new(|| {
    [
        include_bytes!("assets/reference_v2/00.png").as_slice(),
        include_bytes!("assets/reference_v2/01.png").as_slice(),
        include_bytes!("assets/reference_v2/02.png").as_slice(),
        include_bytes!("assets/reference_v2/03.png").as_slice(),
        include_bytes!("assets/reference_v2/04.png").as_slice(),
        include_bytes!("assets/reference_v2/05.png").as_slice(),
        include_bytes!("assets/reference_v2/06.png").as_slice(),
        include_bytes!("assets/reference_v2/07.png").as_slice(),
        include_bytes!("assets/reference_v2/08.png").as_slice(),
        include_bytes!("assets/reference_v2/09.png").as_slice(),
        include_bytes!("assets/reference_v2/10.png").as_slice(),
        include_bytes!("assets/reference_v2/11.png").as_slice(),
        include_bytes!("assets/reference_v2/12.png").as_slice(),
        include_bytes!("assets/reference_v2/13.png").as_slice(),
        include_bytes!("assets/reference_v2/14.png").as_slice(),
        include_bytes!("assets/reference_v2/15.png").as_slice(),
        include_bytes!("assets/reference_v2/16.png").as_slice(),
        include_bytes!("assets/reference_v2/17.png").as_slice(),
        include_bytes!("assets/reference_v2/18.png").as_slice(),
        include_bytes!("assets/reference_v2/19.png").as_slice(),
        include_bytes!("assets/reference_v2/20.png").as_slice(),
        include_bytes!("assets/reference_v2/21.png").as_slice(),
        include_bytes!("assets/reference_v2/22.png").as_slice(),
        include_bytes!("assets/reference_v2/23.png").as_slice(),
        include_bytes!("assets/reference_v2/24.png").as_slice(),
        include_bytes!("assets/reference_v2/25.png").as_slice(),
    ]
    .iter()
    .map(|bytes| decode_png(bytes))
    .collect()
});
