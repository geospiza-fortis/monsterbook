//! Book metadata: tabs, pages, card counts, and ordered entries.
use image::Rgba;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Book {
    pub pages: Vec<Page>,
}

#[derive(Deserialize)]
pub struct Page {
    pub page_id: usize,
    pub tab_color: String,
    pub tab_index: u8,
    pub card_count: usize,
    pub entries: Vec<String>,
}

impl Book {
    /// Cumulative card offsets per page, for computing a card's uid as
    /// offset(page_id) + index within page.
    pub fn offsets(&self) -> Vec<usize> {
        let mut offsets = Vec::with_capacity(self.pages.len() + 1);
        let mut acc = 0;
        offsets.push(acc);
        for page in &self.pages {
            acc += page.card_count;
            offsets.push(acc);
        }
        offsets
    }
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

#[cfg(test)]
mod tests {
    use crate::assets::BOOK;

    #[test]
    fn test_book_pages() {
        assert_eq!(BOOK.pages.len(), 23);
        assert_eq!(BOOK.pages[0].tab_color, "red");
        assert_eq!(BOOK.pages[0].card_count, 13);
        let last = BOOK.pages.last().unwrap();
        assert_eq!(last.tab_color, "gold");
        assert_eq!(last.tab_index, 2);
    }

    #[test]
    fn test_book_entries() {
        let total_cards: usize = BOOK.pages.iter().map(|p| p.card_count).sum();
        let total_entries: usize = BOOK.pages.iter().map(|p| p.entries.len()).sum();
        assert_eq!(total_cards, 418);
        // entries.txt only covers the first 22 pages; the final gold page has
        // no known entries
        assert_eq!(total_entries, 414);
        for page in BOOK.pages.iter().take(22) {
            assert_eq!(page.entries.len(), page.card_count);
        }
        assert_eq!(BOOK.pages[0].entries[0], "Snail");
        assert_eq!(BOOK.pages[21].entries.last().unwrap(), "Giant Centipede");
    }

    #[test]
    fn test_offsets() {
        let offsets = BOOK.offsets();
        assert_eq!(offsets[0], 0);
        assert_eq!(offsets[1], 13);
        assert_eq!(*offsets.last().unwrap(), 418);
    }
}
