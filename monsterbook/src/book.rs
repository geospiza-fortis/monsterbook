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
        // Phase 5: 26 pages, tab colors/counts inferred from the ribbon
        // signal (validated 15/15 against confirmed old->new page mappings,
        // see data/processed/reference_new/REPORT.md) plus order-preserving
        // interpolation for the rest.
        assert_eq!(BOOK.pages.len(), 26);
        assert_eq!(BOOK.pages[0].tab_color, "red");
        assert_eq!(BOOK.pages[0].card_count, 17);
        let last = BOOK.pages.last().unwrap();
        assert_eq!(last.tab_color, "gold");
        assert_eq!(last.tab_index, 3);
    }

    #[test]
    fn test_book_entries() {
        let total_cards: usize = BOOK.pages.iter().map(|p| p.card_count).sum();
        let total_entries: usize = BOOK.pages.iter().map(|p| p.entries.len()).sum();
        // card_count is a lower bound (colored+sketch slots measured via
        // slot_states; placeholder slots can't be distinguished from
        // nonexistent card slots without an authoritative monster list --
        // see data/processed/reference_new/REPORT.md section 3).
        assert_eq!(total_cards, 538);
        assert_eq!(total_entries, 538);
        // every slot has an entries[] cell: either a name carried over from
        // the matched old page, or an "unknown-<uid>" placeholder.
        for page in BOOK.pages.iter() {
            assert_eq!(page.entries.len(), page.card_count);
        }
        assert_eq!(BOOK.pages[0].entries[0], "Snail");
        // page 3 (new, unmapped to any old page) is entirely unknown
        assert!(BOOK.pages[3].entries.iter().all(|e| e.starts_with("unknown-")));
    }

    #[test]
    fn test_offsets() {
        let offsets = BOOK.offsets();
        assert_eq!(offsets[0], 0);
        assert_eq!(offsets[1], 17);
        assert_eq!(*offsets.last().unwrap(), 538);
    }
}
