//! Repro harness: does a small pixel shift (or crop/rescale) of a screenshot
//! break Session ingestion? Run: cargo run --release --example shift_repro <dir>
use image::imageops;
use image::{ImageBuffer, Rgba};
use monsterbook::session::Session;
use monsterbook::vision::Image;

fn translate(img: &Image, dx: i32, dy: i32) -> Image {
    let (w, h) = (img.width() as i32, img.height() as i32);
    ImageBuffer::from_fn(w as u32, h as u32, |x, y| {
        let (sx, sy) = (x as i32 - dx, y as i32 - dy);
        if sx >= 0 && sx < w && sy >= 0 && sy < h {
            *img.get_pixel(sx as u32, sy as u32)
        } else {
            Rgba([0, 0, 0, 255])
        }
    })
}

fn crop_edges(img: &Image, px: u32) -> Image {
    imageops::crop_imm(img, px, px, img.width() - 2 * px, img.height() - 2 * px).to_image()
}

fn rescale(img: &Image, factor: f64) -> Image {
    let (w, h) = (
        (img.width() as f64 * factor).round() as u32,
        (img.height() as f64 * factor).round() as u32,
    );
    imageops::resize(img, w, h, imageops::FilterType::Triangle)
}

fn ingest(img: &Image) -> String {
    let mut s = Session::new();
    match s.add_bitmap(img.width(), img.height(), img.as_raw().clone()) {
        Ok(page_id) => {
            let entries = s.transcribe();
            let total: u32 = entries.iter().map(|e| e.count).sum();
            format!("ok page={page_id} entries={} total_count={total}", entries.len())
        }
        Err(e) => format!("ERR {e}"),
    }
}

fn main() {
    let dir = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "tests".to_string());
    let mut paths: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().map_or(false, |e| e == "png"))
        .collect();
    paths.sort();

    for path in &paths {
        let img = match monsterbook::pipeline::imread(path) {
            Ok(img) => img,
            Err(_) => continue,
        };
        let name = path.file_name().unwrap().to_string_lossy();
        let baseline = ingest(&img);
        println!("{name}");
        println!("  baseline        : {baseline}");
        for (dx, dy) in [(1, 0), (0, 1), (1, 1), (-1, -1), (2, 3), (5, 5)] {
            println!("  shift({dx:+},{dy:+})    : {}", ingest(&translate(&img, dx, dy)));
        }
        for px in [1, 2, 5] {
            println!("  crop_edges({px})   : {}", ingest(&crop_edges(&img, px)));
        }
        for f in [0.99, 1.01, 1.25, 2.0] {
            println!("  rescale({f})   : {}", ingest(&rescale(&img, f)));
        }
    }
}
