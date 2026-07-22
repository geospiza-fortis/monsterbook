//! Composed operations over pages and cards. Functions here take and return
//! values; they never print or save.
use crate::assets;
use crate::book::get_color;
use crate::layout::Layout;
use crate::vision::{self, Image};
use image::ImageError;
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

/// Stitch the non-empty cards into a single image, replacing each card's
/// background with its tab color.
pub fn stitch_cards(pages: &[Image], per_row: u32, layout: &Layout) -> Image {
    let book = &assets::BOOK;
    let cards = extract_cards(pages, layout)
        .into_iter()
        .filter(|card| card.empty_mse() > layout.empty_mse_threshold)
        .map(|card| {
            let color = get_color(&book.pages[card.page_id].tab_color);
            let mut img = card.image;
            vision::replace_background(&mut img, color, layout);
            img
        })
        .collect();
    vision::stitch_images(cards, per_row)
}

/// The empty-card MSE of every card across all pages, in page/grid order.
pub fn empty_card_mse(pages: &[Image], layout: &Layout) -> Vec<u32> {
    extract_cards(pages, layout)
        .iter()
        .map(|card| card.empty_mse())
        .collect()
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
    fn test_reference_page_names() {
        let names = reference_page_names();
        assert_eq!(names.len(), 23);
        assert_eq!(names[0], "00_red_0.png");
        assert_eq!(names[22], "22_gold_2.png");
    }
}
