//! Stitch the non-empty cards of the embedded reference pages into one
//! grid PNG.
use monsterbook::assets::REFERENCE_PAGES_WIN;
use monsterbook::layout::{STITCH_CARDS_PER_ROW, WIN_HD};
use monsterbook::pipeline;
use std::path::PathBuf;

fn main() {
    let output: PathBuf = std::env::args()
        .nth(1)
        .expect("usage: stitch_reference <output.png>")
        .into();
    let stitched = pipeline::stitch_cards(&REFERENCE_PAGES_WIN, STITCH_CARDS_PER_ROW, &WIN_HD);
    pipeline::imsave(&output, &stitched).unwrap();
    println!(
        "wrote {} ({}x{}, {} pages)",
        output.display(),
        stitched.width(),
        stitched.height(),
        REFERENCE_PAGES_WIN.len()
    );
}
