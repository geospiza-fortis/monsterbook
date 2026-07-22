//! Composed operations over pages and cards. Functions here take and return
//! values; they never print or save.
use crate::assets;
use crate::book::get_color;
use crate::layout::Layout;
use crate::book::Book;
use crate::vision::{self, Image};
use image::ImageError;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

pub struct Card {
    pub page_id: usize,
    pub index: usize,
    pub image: Image,
}

impl Card {
    /// MSE of this card against the embedded empty card.
    pub fn empty_mse(&self) -> u32 {
        vision::mse(&self.image, &assets::EMPTY_CARD)
    }
}

/// MSE of an image against the embedded empty card.
pub fn card_mse(img: &Image) -> u32 {
    vision::mse(img, &assets::EMPTY_CARD)
}

/// Find the offset of the embedded reference page within a screenshot.
pub fn match_reference_page(img: &Image) -> (u32, u32) {
    vision::match_reference_page(img, &assets::REFERENCE_PAGE)
}

// TODO(Phase 5+): image 0.23 (via the `image` crate's `webp` feature) can
// only decode *lossless* WebP; screenshots saved as lossy WebP (a plausible
// clipboard/mobile-export source) will fail to decode here. Revisit with a
// newer `image` version (0.24+ added libwebp-based lossy decode) or a
// dedicated `webp` crate if lossy WebP input becomes a real need.
pub fn imread(source: &Path) -> Result<Image, ImageError> {
    Ok(image::io::Reader::open(source)?
        .with_guessed_format()?
        .decode()?
        .into_rgba8())
}

pub fn imsave(output: &Path, img: &Image) -> Result<(), ImageError> {
    img.save(output)
}

/// Read all screenshots in a directory (sorted by filename, since these are
/// zipped against page metadata) and crop each one to a page.
pub fn load_pages(source: &Path, layout: &Layout) -> Result<Vec<Image>, ImageError> {
    let mut paths: Vec<PathBuf> = fs::read_dir(source)?
        .map(|entry| entry.map(|e| e.path()))
        .collect::<Result<_, _>>()?;
    paths.sort();
    let mut images = Vec::new();
    let (mut x, mut y) = (0, 0);
    for path in paths {
        let mut img = imread(&path)?;
        if x == 0 && y == 0 {
            let (a, b) = match_reference_page(&img);
            x = a;
            y = b;
        }
        images.push(vision::crop_page(&mut img, x, y, layout));
    }
    Ok(images)
}

/// Split cropped pages into cards, tagged with their page and index.
pub fn extract_cards(pages: &[Image], layout: &Layout) -> Vec<Card> {
    pages
        .iter()
        .enumerate()
        .flat_map(|(page_id, page)| {
            vision::crop_cards(page, layout)
                .into_iter()
                .enumerate()
                .map(move |(index, image)| Card {
                    page_id,
                    index,
                    image,
                })
        })
        .collect()
}

/// Stitch the non-empty cards of (page_id, page) pairs into a single image,
/// replacing each card's background with the page's tab color. Returns None
/// when there are no non-empty cards to stitch.
pub fn stitch_page_cards<'a, I>(pages: I, per_row: u32, layout: &Layout) -> Option<Image>
where
    I: IntoIterator<Item = (usize, &'a Image)>,
{
    let book = &assets::BOOK;
    let cards: Vec<Image> = pages
        .into_iter()
        .flat_map(|(page_id, page)| {
            vision::crop_cards(page, layout)
                .into_iter()
                .map(move |image| (page_id, image))
        })
        .filter(|(_, image)| card_mse(image) > layout.empty_mse_threshold)
        .map(|(page_id, mut image)| {
            let color = get_color(&book.pages[page_id].tab_color);
            vision::replace_background(&mut image, color, layout);
            image
        })
        .collect();
    if cards.is_empty() {
        return None;
    }
    Some(vision::stitch_images(cards, per_row))
}

/// Stitch the non-empty cards into a single image, replacing each card's
/// background with its tab color. Pages are assumed to be in book order
/// starting at page 0.
pub fn stitch_cards(pages: &[Image], per_row: u32, layout: &Layout) -> Image {
    stitch_page_cards(pages.iter().enumerate(), per_row, layout)
        .unwrap_or_else(|| Image::new(0, 0))
}

/// The empty-card MSE of every card across all pages, in page/grid order.
pub fn empty_card_mse(pages: &[Image], layout: &Layout) -> Vec<u32> {
    extract_cards(pages, layout)
        .iter()
        .map(|card| card.empty_mse())
        .collect()
}

/// A transcribed monster book entry.
#[derive(Serialize, Debug)]
pub struct Entry {
    pub uid: usize,
    pub name: String,
    pub count: u32,
}

/// Index of the closest reference image by minimum MSE (and that MSE),
/// mirroring python/utils.py `match`.
pub fn match_min_mse(img: &Image, refs: &[Image]) -> (usize, u32) {
    refs.iter()
        .enumerate()
        .map(|(i, r)| (i, vision::mse(img, r)))
        .min_by_key(|(_, m)| *m)
        .unwrap()
}

/// Index of the closest reference image by maximum zero-mean normalized
/// cross-correlation (and that correlation, in [-1, 1]). Used for page
/// identification instead of `match_min_mse`: it is invariant to the small
/// global brightness/contrast differences between screenshots, which raw
/// grayscale MSE is not.
pub fn match_max_ncc(img: &Image, refs: &[Image]) -> (usize, f64) {
    refs.iter()
        .enumerate()
        .map(|(i, r)| (i, vision::ncc(img, r)))
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .unwrap()
}

/// The entry name for a uid, or a placeholder for uids past the known
/// entries (the final gold page has no known entry names).
fn entry_name(book: &Book, offsets: &[usize], uid: usize) -> String {
    for (page_id, page) in book.pages.iter().enumerate() {
        if uid >= offsets[page_id] && uid < offsets[page_id + 1] {
            if let Some(name) = page.entries.get(uid - offsets[page_id]) {
                return name.clone();
            }
        }
    }
    format!("unknown-{}", uid)
}

/// Transcribe cropped pages into entries, mirroring python/cli.py
/// `transcribe`: each page is matched to the embedded reference set by
/// minimum grayscale MSE, empty cards are dropped, and each remaining card's
/// count is read from its tag against the seed digits (or 0 if the card is
/// registered but unseen).
pub fn transcribe(pages: &[Image], book: &Book, layout: &Layout) -> Vec<Entry> {
    let refs = &assets::REFERENCE_PAGES_WIN;
    let mut data = Vec::new();
    for page in pages {
        let (index, _) = match_min_mse(page, refs);
        data.extend(transcribe_page(page, index, book, layout));
    }
    data
}

/// Transcribe a single cropped page whose page_id is already known.
pub fn transcribe_page(page: &Image, page_id: usize, book: &Book, layout: &Layout) -> Vec<Entry> {
    let seeds = &assets::SEED_TAGS;
    let empty = &assets::EMPTY_CARD;
    let offsets = book.offsets();
    let cards: Vec<Image> = vision::crop_cards(page, layout)
        .into_iter()
        .filter(|card| vision::mse(card, empty) > layout.empty_mse_threshold)
        .collect();
    let mut data = Vec::new();
    for (i, card) in cards.iter().enumerate() {
        let uid = offsets[page_id] + i;
        let mut count = 0;
        if vision::mse(card, empty) > layout.unseen_mse_threshold {
            let tag = vision::crop_tag(card, layout);
            count = match_min_mse(&tag, seeds).0 as u32 + 1;
        }
        data.push(Entry {
            uid,
            name: entry_name(book, &offsets, uid),
            count,
        });
    }
    data
}

/// Filenames for the reference-book command, one per page in book order.
pub fn reference_page_names() -> Vec<String> {
    assets::BOOK
        .pages
        .iter()
        .map(|page| format!("{:02}_{}_{}.png", page.page_id, page.tab_color, page.tab_index))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::WIN_HD;
    use image::RgbaImage;

    #[test]
    fn test_extract_cards_indices() {
        let pages = vec![RgbaImage::new(165, 225), RgbaImage::new(165, 225)];
        let cards = extract_cards(&pages, &WIN_HD);
        assert_eq!(cards.len(), 50);
        assert_eq!(cards[0].page_id, 0);
        assert_eq!(cards[24].index, 24);
        assert_eq!(cards[25].page_id, 1);
        assert_eq!(cards[25].index, 0);
        assert_eq!(cards[0].image.width(), 33);
        assert_eq!(cards[0].image.height(), 45);
    }

    #[test]
    fn test_transcribe_reference_pages() {
        // transcribing the reference book against itself: every page matches
        // its own index, and a mostly-unregistered book yields few entries
        let pages: Vec<Image> = assets::REFERENCE_PAGES_WIN.iter().cloned().collect();
        let entries = transcribe(&pages, &assets::BOOK, &WIN_HD);
        assert!(entries.len() <= 538);
        let offsets = assets::BOOK.offsets();
        for entry in &entries {
            assert!(entry.uid < *offsets.last().unwrap());
            assert!(!entry.name.is_empty());
            assert!(entry.count <= 5);
        }
        // the first page of the reference book has a seen snail
        assert!(entries.iter().any(|e| e.name == "Snail"));
    }

    #[test]
    fn test_reference_page_names() {
        let names = reference_page_names();
        assert_eq!(names.len(), 26);
        assert_eq!(names[0], "00_red_0.png");
        assert_eq!(names[25], "25_gold_3.png");
    }
}
