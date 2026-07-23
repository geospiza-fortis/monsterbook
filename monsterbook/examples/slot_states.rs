//! Classify each of the 25 grid slots on every dumped reference_new page
//! into: placeholder (leaf sparkle, ~matches EMPTY_CARD), sketch (low
//! saturation content: greyscale outline), or colored (normal saturated
//! card art). This is a measurable proxy only -- it does NOT attempt to
//! infer "true" card_count, since sketch vs colored vs placeholder
//! semantics in a personal book are ambiguous (see REPORT.md).
use monsterbook::layout::WIN_HD;
use monsterbook::{assets, pipeline, vision};
use std::path::PathBuf;

fn saturation(img: &vision::Image) -> f64 {
    // mean of (max-min)/255 over RGB channels, restricted to the content
    // rect so the card border/background doesn't dilute the signal
    let l = &WIN_HD;
    let cropped = image::imageops::crop_imm(img, l.content_x, l.content_y, l.content_width, l.content_height)
        .to_image();
    let mut acc = 0.0;
    let mut n = 0u32;
    for p in cropped.pixels() {
        let (r, g, b) = (p[0] as i32, p[1] as i32, p[2] as i32);
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        acc += (max - min) as f64;
        n += 1;
    }
    acc / n as f64
}

fn main() {
    let dir: PathBuf = std::env::args().nth(1).unwrap().into();
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    paths.sort();

    for path in paths {
        let page = pipeline::imread(&path).unwrap();
        let cards = vision::crop_cards(&page, &WIN_HD);
        let mut placeholder = 0;
        let mut sketch = 0;
        let mut colored = 0;
        let mut row_summary = String::new();
        for (i, card) in cards.iter().enumerate() {
            let empty_mse = vision::mse(card, &assets::EMPTY_CARD);
            let state = if empty_mse <= WIN_HD.empty_mse_threshold {
                placeholder += 1;
                '.'
            } else if saturation(card) < 12.0 {
                sketch += 1;
                's'
            } else {
                colored += 1;
                'C'
            };
            row_summary.push(state);
            if i % 5 == 4 {
                row_summary.push(' ');
            }
        }
        println!(
            "{}: colored={} sketch={} placeholder={} [{}]",
            path.file_name().unwrap().to_string_lossy(),
            colored,
            sketch,
            placeholder,
            row_summary.trim()
        );
    }
}
