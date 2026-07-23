//! Content-addressed screenshot ingestion: screenshots are identified by
//! which book page they contain rather than by filename order, so they can be
//! added in any order, from any source (files, drag-and-drop, clipboard).
use crate::assets;
use crate::layout::{Layout, WIN_HD};
use crate::pipeline::{self, Entry};
use crate::vision::{self, Image};
use std::collections::BTreeMap;
use std::fmt;
use std::io::Cursor;

/// Why a screenshot could not be ingested.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IngestError {
    /// The bytes could not be decoded as an image.
    DecodeError,
    /// No monster book page was found in the image (the best reference-page
    /// match exceeded `Layout::page_mse_threshold`).
    NoPageFound,
}

impl fmt::Display for IngestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IngestError::DecodeError => write!(f, "could not decode image"),
            IngestError::NoPageFound => write!(f, "no monster book page found in image"),
        }
    }
}

impl std::error::Error for IngestError {}

/// A collection of cropped book pages keyed by page_id, built up
/// incrementally from screenshots.
pub struct Session {
    pages: BTreeMap<usize, Image>,
    layout: &'static Layout,
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

impl Session {
    pub fn new() -> Self {
        Session {
            pages: BTreeMap::new(),
            layout: &WIN_HD,
        }
    }

    /// The ingested pages, keyed by page_id.
    pub fn pages(&self) -> &BTreeMap<usize, Image> {
        &self.pages
    }

    /// Decode a screenshot from raw file bytes (format guessed) and ingest it.
    /// Returns the identified page_id.
    pub fn add_screenshot(&mut self, bytes: &[u8]) -> Result<usize, IngestError> {
        let img = image::io::Reader::new(Cursor::new(bytes))
            .with_guessed_format()
            .map_err(|_| IngestError::DecodeError)?
            .decode()
            .map_err(|_| IngestError::DecodeError)?
            .into_rgba8();
        self.ingest(img)
    }

    /// Ingest a raw RGBA bitmap (e.g. from the clipboard). Returns the
    /// identified page_id.
    pub fn add_bitmap(
        &mut self,
        width: u32,
        height: u32,
        rgba: Vec<u8>,
    ) -> Result<usize, IngestError> {
        let img = Image::from_raw(width, height, rgba).ok_or(IngestError::DecodeError)?;
        self.ingest(img)
    }

    /// Locate and crop the book page, identify which page it is by minimum
    /// grayscale MSE against the embedded reference pages, and insert it
    /// (replacing any previous screenshot of the same page).
    fn ingest(&mut self, mut img: Image) -> Result<usize, IngestError> {
        if img.width() < self.layout.page_width || img.height() < self.layout.page_height {
            return Err(IngestError::NoPageFound);
        }
        let (x, y) = vision::match_reference_page(&img, &assets::REFERENCE_PAGE);
        let page = vision::crop_page(&mut img, x, y, self.layout);
        if page.width() != self.layout.page_width || page.height() != self.layout.page_height {
            return Err(IngestError::NoPageFound);
        }
        let (page_id, best_ncc) = pipeline::match_max_ncc(&page, &assets::REFERENCE_PAGES_WIN);
        if best_ncc < self.layout.page_ncc_threshold {
            return Err(IngestError::NoPageFound);
        }
        self.pages.insert(page_id, page);
        Ok(page_id)
    }

    /// Merge another session's pages into this one; the other session's
    /// pages win on conflict (they are newer).
    pub fn merge(&mut self, other: Session) {
        self.pages.extend(other.pages);
    }

    /// Page ids not yet ingested. All 26 pages now have an embedded
    /// reference screenshot (`assets::REFERENCE_PAGES_WIN`), so none are
    /// permanently excluded (unlike the pre-Phase-5 22-page set, which had
    /// no reference for the final gold page).
    pub fn missing(&self) -> Vec<usize> {
        (0..assets::REFERENCE_PAGES_WIN.len())
            .filter(|id| !self.pages.contains_key(id))
            .collect()
    }

    /// Transcribe only the ingested pages, using each page's identified
    /// page_id directly (no re-matching).
    pub fn transcribe(&self) -> Vec<Entry> {
        self.pages
            .iter()
            .flat_map(|(&page_id, page)| {
                pipeline::transcribe_page(page, page_id, &assets::BOOK, self.layout)
            })
            .collect()
    }

    /// Stitch the cards of the ingested pages, in page order. Empty
    /// (un-caught) card slots are skipped unless `include_empty` is set.
    /// None when the session is empty (or has no cards to stitch).
    pub fn stitch(&self, cards_per_row: u32, include_empty: bool) -> Option<Image> {
        pipeline::stitch_page_cards(
            self.pages.iter().map(|(&id, page)| (id, page)),
            cards_per_row,
            include_empty,
            self.layout,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, ImageOutputFormat, Rgba, RgbaImage};

    /// Build a synthetic screenshot: the win-scale reference page embedded
    /// at an offset in a larger canvas, re-encoded as PNG bytes.
    fn encoded_reference(page_id: usize) -> Vec<u8> {
        let page = &assets::REFERENCE_PAGES_WIN[page_id];
        let mut canvas = RgbaImage::from_pixel(400, 300, Rgba([20, 20, 20, 255]));
        image::imageops::overlay(&mut canvas, page, 40, 30);
        let mut bytes = Cursor::new(Vec::new());
        DynamicImage::ImageRgba8(canvas)
            .write_to(&mut bytes, ImageOutputFormat::Png)
            .unwrap();
        bytes.into_inner()
    }

    #[test]
    fn test_add_screenshot_identifies_page() {
        let mut session = Session::new();
        let page_id = session.add_screenshot(&encoded_reference(3)).unwrap();
        assert_eq!(page_id, 3);
        assert_eq!(session.pages().len(), 1);
    }

    #[test]
    fn test_add_screenshot_bad_bytes() {
        let mut session = Session::new();
        assert_eq!(
            session.add_screenshot(b"not an image"),
            Err(IngestError::DecodeError)
        );
    }

    #[test]
    fn test_solid_image_rejected() {
        let mut session = Session::new();
        let solid = RgbaImage::from_pixel(400, 300, Rgba([192, 192, 192, 255]));
        let result = session.add_bitmap(400, 300, solid.into_raw());
        assert_eq!(result, Err(IngestError::NoPageFound));
        assert!(session.pages().is_empty());
    }

    #[test]
    fn test_add_bitmap_identifies_page() {
        // pad the page into a larger canvas (as a real screenshot would
        // have surrounding UI chrome) so phase correlation against the
        // single-page anchor has real structure to lock onto; a bare
        // page-sized bitmap with no surrounding context is not a realistic
        // input and phase-correlates unreliably.
        let mut session = Session::new();
        let page = &assets::REFERENCE_PAGES_WIN[0];
        let mut canvas = RgbaImage::from_pixel(400, 300, Rgba([20, 20, 20, 255]));
        image::imageops::overlay(&mut canvas, page, 40, 30);
        let (w, h) = (canvas.width(), canvas.height());
        let page_id = session.add_bitmap(w, h, canvas.into_raw()).unwrap();
        assert_eq!(page_id, 0);
    }

    #[test]
    fn test_duplicate_add_replaces() {
        let mut session = Session::new();
        session.add_screenshot(&encoded_reference(5)).unwrap();
        session.add_screenshot(&encoded_reference(5)).unwrap();
        assert_eq!(session.pages().len(), 1);
    }

    #[test]
    fn test_missing_shrinks() {
        let mut session = Session::new();
        let missing = session.missing();
        // all 26 pages are identifiable now
        assert_eq!(missing.len(), 26);
        session.add_screenshot(&encoded_reference(0)).unwrap();
        let missing = session.missing();
        assert_eq!(missing.len(), 25);
        assert!(!missing.contains(&0));
    }

    #[test]
    fn test_partial_transcribe() {
        // the reference pages are crops of an empty book, so an ingested
        // reference page transcribes to no entries (nothing registered)
        let mut session = Session::new();
        session.add_screenshot(&encoded_reference(0)).unwrap();
        assert!(session.transcribe().is_empty());
    }

    #[test]
    fn test_page_mse_threshold_margins() {
        // the calibration behind Layout::page_mse_threshold: a genuine page
        // matches its reference near 0, while non-book images stay well
        // above the threshold.
        let refs = &assets::REFERENCE_PAGES_WIN;
        let threshold = WIN_HD.page_mse_threshold;
        for r in refs.iter() {
            let (_, best) = pipeline::match_min_mse(r, refs);
            assert!(best < threshold / 2);
        }
        for v in [0u8, 64, 128, 192, 255] {
            let solid = RgbaImage::from_pixel(165, 225, Rgba([v, v, v, 255]));
            let (_, best) = pipeline::match_min_mse(&solid, refs);
            assert!(best > threshold * 2, "solid {} mse {}", v, best);
        }
    }

    #[test]
    fn test_page_ncc_threshold_margins() {
        // the calibration behind Layout::page_ncc_threshold: a genuine page
        // matches its reference near 1.0, while non-book images (constant
        // fill or noise, which have ~zero variance/structure) score far
        // below the threshold.
        let refs = &assets::REFERENCE_PAGES_WIN;
        let threshold = WIN_HD.page_ncc_threshold;
        for r in refs.iter() {
            let (_, best) = pipeline::match_max_ncc(r, refs);
            assert!(best > 0.99, "self-match ncc {}", best);
        }
        for v in [0u8, 64, 128, 192, 255] {
            let solid = RgbaImage::from_pixel(165, 225, Rgba([v, v, v, 255]));
            let (_, best) = pipeline::match_max_ncc(&solid, refs);
            assert!(best < threshold, "solid {} ncc {}", v, best);
        }
        // pseudo-random noise, deterministic so the test is reproducible
        let mut state: u32 = 12345;
        let noise = RgbaImage::from_fn(165, 225, |_, _| {
            state = state.wrapping_mul(1103515245).wrapping_add(12345);
            let v = ((state >> 16) & 0xff) as u8;
            Rgba([v, v, v, 255])
        });
        let (_, best) = pipeline::match_max_ncc(&noise, refs);
        assert!(best < threshold, "noise ncc {}", best);
    }

    #[test]
    fn test_stitch() {
        let mut session = Session::new();
        assert!(session.stitch(10, false).is_none());
        // an ingested empty reference page has no non-empty cards either
        session.add_screenshot(&encoded_reference(0)).unwrap();
        assert!(session.stitch(10, false).is_none());
        // ...unless empty slots are explicitly included
        assert!(session.stitch(10, true).is_some());
        // paint one card slot with a bright square so it stitches
        let mut page = assets::REFERENCE_PAGES_WIN[0].clone();
        for y in 0..45 {
            for x in 0..33 {
                page.put_pixel(x, y, Rgba([255, 0, 0, 255]));
            }
        }
        session.pages.insert(0, page);
        let stitched = session.stitch(10, false).unwrap();
        assert!(stitched.width() > 0 && stitched.height() > 0);
    }
}
