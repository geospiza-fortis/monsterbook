//! Diagnose a single screenshot: print the top phase-correlation candidate
//! offsets and their best NCC against the reference pages.
//! Usage: user_probe <image> [out_crop.png]
use monsterbook::{assets, layout::WIN_HD, pipeline, vision};

fn main() {
    let path = std::env::args().nth(1).expect("usage: user_probe <image>");
    let mut img = pipeline::imread(std::path::Path::new(&path)).expect("decode");
    println!("image: {}x{}", img.width(), img.height());
    let candidates = vision::match_reference_page_candidates(&img, &assets::REFERENCE_PAGE, 10);
    let mut best: Option<(u32, u32, usize, f64)> = None;
    for (x, y) in candidates {
        let page = vision::crop_page(&mut img, x, y, &WIN_HD);
        if page.width() != WIN_HD.page_width || page.height() != WIN_HD.page_height {
            println!("candidate ({x}, {y}): crop {}x{} (skipped)", page.width(), page.height());
            continue;
        }
        let (page_id, ncc) = pipeline::match_max_ncc(&page, &assets::REFERENCE_PAGES_WIN);
        println!("candidate ({x}, {y}): page {page_id} ncc {ncc:.4}");
        if best.map_or(true, |(_, _, _, b)| ncc > b) {
            best = Some((x, y, page_id, ncc));
        }
    }
    let Some((x, y, page_id, ncc)) = best else {
        println!("no viable candidate");
        return;
    };
    println!("best: ({x}, {y}) page {page_id} ncc {ncc:.4}");
    let page = vision::crop_page(&mut img, x, y, &WIN_HD);
    let mut scores: Vec<(usize, f64)> = assets::REFERENCE_PAGES_WIN
        .iter()
        .enumerate()
        .map(|(i, r)| (i, vision::ncc(&page, r)))
        .collect();
    scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    for (i, s) in scores.iter().take(5) {
        println!("page {i}: ncc {s:.4}");
    }
    if let Some(out) = std::env::args().nth(2) {
        pipeline::imsave(std::path::Path::new(&out), &page).unwrap();
        println!("wrote {out}");
    }
}
