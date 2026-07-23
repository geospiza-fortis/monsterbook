/// All pixel geometry and thresholds for a particular client resolution.
pub struct Layout {
    /// Size of a cropped monster book page.
    pub page_width: u32,
    pub page_height: u32,
    /// Cards are arranged in a grid on each page.
    pub grid_rows: u32,
    pub grid_cols: u32,
    /// The card content rect, relative to a card; see notebook, but we go
    /// from [4:-3, 3:-3] in numpy.
    pub content_x: u32,
    pub content_y: u32,
    pub content_width: u32,
    pub content_height: u32,
    /// MSE against the empty card below which a card is considered empty.
    /// Calibrated on the 650 known-empty slots of the 26 empty-book pages
    /// (`data/processed/empty_new/`) against the mean empty-card template:
    /// every empty slot scores at most 1010, while the closest non-empty
    /// card in the collected reference set
    /// (`data/processed/reference_new/`) scores 4690. 2000 leaves roughly
    /// 2x margin on both sides.
    pub empty_mse_threshold: u32,
    /// The count tag rect (the small "x<n>" badge in the card's lower left),
    /// relative to a card. From python/utils.py `crop_tag_win`: numpy
    /// `card[31:31+9, 5:5+6]`, i.e. row (y) 31, col (x) 5, 9 rows tall and 6
    /// cols wide.
    pub tag_x: u32,
    pub tag_y: u32,
    pub tag_width: u32,
    pub tag_height: u32,
    /// MSE against the empty card above which a card is considered "seen"
    /// (its count tag is legible); at or below, the monster is registered but
    /// unseen and its count is 0. The collected reference set contains no
    /// unseen (silhouette) cards, so this cannot be calibrated directly;
    /// instead it mirrors the old threshold's relative placement. Against
    /// the old template the seen cluster started at 7732 and the python-era
    /// UNSEEN_THRESHOLD of 5000 sat at ~0.65x that; against the new
    /// template the seen cluster starts at 4690, and 3000 sits at the same
    /// ~0.65x while staying well above the empty cluster's 1010 maximum.
    pub unseen_mse_threshold: u32,
    /// Grayscale MSE against the closest embedded reference page above which
    /// a cropped page is rejected as "not a monster book page". Calibrated
    /// empirically against the empty-book reference set: a reference page
    /// matched against itself is 0, while the closest non-book image tried
    /// (a solid white fill, close to the empty pages' beige) scores 920 and
    /// random noise scores ~11900. 300 keeps 3x margin below the closest
    /// non-book input.
    ///
    /// Deprecated by `page_ncc_threshold` for page *identification*, but
    /// still kept (and still tested) as it remains a reasonable
    /// page-vs-noise rejection rule and existing callers (`Transcribe`,
    /// `ReferenceBook`) still key off the embedded `REFERENCE_PAGES*` MSE
    /// tables.
    pub page_mse_threshold: u32,
    /// Zero-mean normalized cross-correlation (in [-1, 1]) against the
    /// closest embedded reference page below which a cropped page is
    /// rejected as "not a monster book page". Calibrated against the
    /// empty-book reference set on the 26 fully collected-book pages
    /// (`data/processed/reference_new/`, the worst-case card-content
    /// mismatch): every real page identifies correctly with NCC of at least
    /// 0.745, while a solid fill or random noise image scores ~0.0 against
    /// every reference (NCC is undefined/zero for a constant image). 0.3
    /// leaves comfortable margin on both sides while still being decisive
    /// against non-book input.
    pub page_ncc_threshold: f64,
}

/// Cards per row when stitching all non-empty cards into one image.
pub const STITCH_CARDS_PER_ROW: u32 = 24;

pub const WIN_HD: Layout = Layout {
    page_width: 165,
    page_height: 225,
    grid_rows: 5,
    grid_cols: 5,
    content_x: 3,
    content_y: 4,
    content_width: 27,
    content_height: 38,
    empty_mse_threshold: 2000,
    tag_x: 5,
    tag_y: 31,
    tag_width: 6,
    tag_height: 9,
    unseen_mse_threshold: 3000,
    page_mse_threshold: 300,
    page_ncc_threshold: 0.3,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_win_hd_dimensions() {
        assert_eq!(WIN_HD.page_width, 165);
        assert_eq!(WIN_HD.page_height, 225);
        assert_eq!(WIN_HD.grid_rows * WIN_HD.grid_cols, 25);
        assert_eq!(WIN_HD.empty_mse_threshold, 2000);
    }

    #[test]
    fn test_win_hd_tag_rect() {
        // python/utils.py crop_tag_win: card[31:40, 5:11]
        assert_eq!(WIN_HD.tag_x, 5);
        assert_eq!(WIN_HD.tag_y, 31);
        assert_eq!(WIN_HD.tag_width, 6);
        assert_eq!(WIN_HD.tag_height, 9);
        // the tag must fit within a 33x45 card
        assert!(WIN_HD.tag_x + WIN_HD.tag_width <= 165 / WIN_HD.grid_cols);
        assert!(WIN_HD.tag_y + WIN_HD.tag_height <= 225 / WIN_HD.grid_rows);
        assert_eq!(WIN_HD.unseen_mse_threshold, 3000);
    }
}
