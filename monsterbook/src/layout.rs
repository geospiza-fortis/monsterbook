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
    /// MSE against the empty card below which a card is considered empty. To
    /// determine the threshold, generate stats and look for an obvious cutoff.
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
    /// unseen and its count is 0. From python/cli.py `transcribe`
    /// UNSEEN_THRESHOLD.
    pub unseen_mse_threshold: u32,
}

pub const WIN_HD: Layout = Layout {
    page_width: 165,
    page_height: 225,
    grid_rows: 5,
    grid_cols: 5,
    content_x: 3,
    content_y: 4,
    content_width: 27,
    content_height: 38,
    empty_mse_threshold: 500,
    tag_x: 5,
    tag_y: 31,
    tag_width: 6,
    tag_height: 9,
    unseen_mse_threshold: 5000,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_win_hd_dimensions() {
        assert_eq!(WIN_HD.page_width, 165);
        assert_eq!(WIN_HD.page_height, 225);
        assert_eq!(WIN_HD.grid_rows * WIN_HD.grid_cols, 25);
        assert_eq!(WIN_HD.empty_mse_threshold, 500);
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
        assert_eq!(WIN_HD.unseen_mse_threshold, 5000);
    }
}
