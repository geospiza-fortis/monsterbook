//! Programmatic ribbon tab-color reader: for each screenshot, crop the
//! flag-strip left of the anchor (extended past page_height to include the
//! gold band), measure the rightmost "flag-colored" pixel column within
//! each of the 8 saturation-detectable band row-ranges (black's flag is a
//! low-saturation gray and is not directly detectable this way), and report
//! the extended (active) band as the one whose max extent clears a margin
//! over the rest. If no band clears the margin, infer "black" by
//! elimination.
use monsterbook::pipeline;
use std::path::PathBuf;

// (name, y0, y1) row ranges within the strip, in strip-relative pixels,
// calibrated against data/processed/reference_new/REPORT.md ribbon dumps.
const BANDS: [(&str, u32, u32); 9] = [
    ("red", 75, 84),
    ("orange", 96, 105),
    ("lightgreen", 114, 126),
    ("green", 135, 144),
    ("lightblue", 156, 165),
    ("blue", 174, 186),
    ("purple", 195, 204),
    ("black", 216, 225),
    ("gold", 234, 246),
];

fn row_extent(strip: &monsterbook::vision::Image, y0: u32, y1: u32) -> i32 {
    let mut last = -1i32;
    for yy in y0..y1.min(strip.height()) {
        for xx in 0..strip.width() {
            let p = strip.get_pixel(xx, yy);
            let (r, g, b) = (p[0] as i32, p[1] as i32, p[2] as i32);
            let mx = r.max(g).max(b);
            let mn = r.min(g).min(b);
            if mx - mn > 60 && mx < 250 {
                last = last.max(xx as i32);
            }
        }
    }
    last
}

fn main() {
    let source: PathBuf = std::env::args().nth(1).unwrap().into();
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&source)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    paths.sort();

    for (i, path) in paths.iter().enumerate() {
        let mut img = pipeline::imread(path).unwrap();
        let (x, y) = pipeline::match_reference_page(&img);
        let strip_w = x.min(60);
        let strip_x = x - strip_w;
        let strip_h = 270u32.min(img.height() - y);
        let strip = image::imageops::crop(&mut img, strip_x, y, strip_w, strip_h).to_image();

        let extents: Vec<i32> = BANDS.iter().map(|(_, y0, y1)| row_extent(&strip, *y0, *y1)).collect();
        let (best_i, &best_v) = extents.iter().enumerate().max_by_key(|(_, v)| **v).unwrap();
        let second = extents.iter().enumerate().filter(|(i,_)| *i != best_i).map(|(_,v)| *v).max().unwrap();
        let color = if best_v - second < 8 {
            "black" // no band clearly extended -> infer by elimination
        } else {
            BANDS[best_i].0
        };
        println!(
            "{:02} {}: extents={:?} best={}({}) margin={} -> {}",
            i,
            path.file_name().unwrap().to_string_lossy(),
            extents,
            BANDS[best_i].0,
            best_v,
            best_v - second,
            color
        );
    }
}
