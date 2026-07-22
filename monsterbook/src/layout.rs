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
}
