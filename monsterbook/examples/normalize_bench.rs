//! Calibration experiment for
//! https://github.com/geospiza-fortis/monsterbook/issues/2 : find a
//! per-image normalization that makes a COLLECTED (colored) card and its
//! SKETCH rendering (seen-but-uncollected grayscale line art of the same
//! artwork) score alike, so page/card identification is robust across
//! collection states.
//!
//! Card-level test pairs are built from the 15 confirmed
//! screenshot-index -> old-reference-page mappings (see
//! `data/processed/reference_new/REPORT.md` section 2): for each pair, crop
//! both pages into 25 slots and keep the slots where the new (heavily
//! collected) card is COLORED and the corresponding old-reference slot is a
//! SKETCH -- these are the (colored, own-sketch) test pairs. Every
//! normalization candidate is scored on: (1) card-level top-1 accuracy under
//! NCC (does the colored card's own sketch beat every *other* sketch in the
//! test set?) and mean margin; (2) page-level NCC margin (26 screenshots vs.
//! 22 old references), restricted to the 15 confirmed pages, to check the
//! candidate doesn't regress the existing page-identification margins.
//!
//! Run with: cargo run --no-default-features --example normalize_bench
use monsterbook::layout::WIN_HD;
use monsterbook::{assets, pipeline, vision};
use ndarray::Array2;
use std::path::PathBuf;

/// screenshot dump index (data/processed/reference_new/<idx>.png) -> old
/// REFERENCE_PAGES_WIN index, confirmed/moderate confidence mappings from
/// REPORT.md section 2.
const CONFIRMED_MAPPING: &[(usize, usize)] = &[
    (0, 0),
    (1, 1),
    (2, 2),
    (4, 4),
    (5, 5),
    (6, 6),
    (9, 8),
    (10, 9),
    (13, 11),
    (14, 12),
    (16, 14),
    (18, 16),
    (20, 18),
    (22, 20),
    (23, 21),
];

// ---------------------------------------------------------------------
// slot-state classification (mirrors examples/slot_states.rs)
// ---------------------------------------------------------------------

fn saturation(img: &vision::Image) -> f64 {
    let l = &WIN_HD;
    let cropped = image::imageops::crop_imm(
        img,
        l.content_x,
        l.content_y,
        l.content_width,
        l.content_height,
    )
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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum SlotState {
    Placeholder,
    Sketch,
    Colored,
}

fn classify(card: &vision::Image) -> SlotState {
    let empty_mse = vision::mse(card, &assets::EMPTY_CARD);
    if empty_mse <= WIN_HD.empty_mse_threshold {
        SlotState::Placeholder
    } else if saturation(card) < 12.0 {
        SlotState::Sketch
    } else {
        SlotState::Colored
    }
}

// ---------------------------------------------------------------------
// normalization candidates: Image -> Array2<f64>, same-size in/out
// ---------------------------------------------------------------------

fn to_gray_f64(img: &vision::Image) -> Array2<f64> {
    vision::into_grayscale_array(img).mapv(|x| x as f64)
}

/// A. baseline: plain grayscale, no normalization.
fn norm_baseline(img: &vision::Image) -> Array2<f64> {
    to_gray_f64(img)
}

/// B. grayscale + per-image 2nd/98th percentile contrast stretch to [0,255].
fn norm_contrast_stretch(img: &vision::Image) -> Array2<f64> {
    let gray = to_gray_f64(img);
    let mut sorted: Vec<f64> = gray.iter().cloned().collect();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = sorted.len();
    let lo = sorted[(n as f64 * 0.02) as usize];
    let hi = sorted[((n as f64 * 0.98) as usize).min(n - 1)];
    let range = (hi - lo).max(1e-6);
    gray.mapv(|x| ((x - lo) / range * 255.0).clamp(0.0, 255.0))
}

/// C. grayscale + per-image histogram equalization.
fn norm_hist_eq(img: &vision::Image) -> Array2<f64> {
    let gray = vision::into_grayscale_array(img);
    let mut hist = [0u32; 256];
    for &v in gray.iter() {
        hist[v as usize] += 1;
    }
    let n = gray.len() as f64;
    let mut cdf = [0.0f64; 256];
    let mut acc = 0u32;
    for i in 0..256 {
        acc += hist[i];
        cdf[i] = acc as f64 / n;
    }
    gray.mapv(|v| cdf[v as usize] * 255.0)
}

/// D. Sobel gradient magnitude (vision::sobel_magnitude) + per-image
/// max-normalization to [0,255].
fn norm_sobel(img: &vision::Image) -> Array2<f64> {
    let mag = vision::sobel_magnitude(img).mapv(|x| x as f64);
    let max = mag.iter().cloned().fold(0.0f64, f64::max).max(1e-6);
    mag.mapv(|x| x / max * 255.0)
}

/// E. adaptive binarization: pixel < (global mean - k) -> 0, else 255.
fn norm_adaptive_binary(img: &vision::Image) -> Array2<f64> {
    let gray = to_gray_f64(img);
    let mean = gray.mean().unwrap();
    let k = 10.0;
    let threshold = mean - k;
    gray.mapv(|x| if x < threshold { 0.0 } else { 255.0 })
}

/// Zero-mean NCC directly on two same-shape f64 arrays.
fn ncc_arr(a: &Array2<f64>, b: &Array2<f64>) -> f64 {
    let mean_a = a.mean().unwrap();
    let mean_b = b.mean().unwrap();
    let mut num = 0.0;
    let mut da = 0.0;
    let mut db = 0.0;
    for (x, y) in a.iter().zip(b.iter()) {
        let dx = x - mean_a;
        let dy = y - mean_b;
        num += dx * dy;
        da += dx * dx;
        db += dy * dy;
    }
    if da == 0.0 || db == 0.0 {
        return 0.0;
    }
    num / (da.sqrt() * db.sqrt())
}

struct Candidate {
    name: &'static str,
    normalize: fn(&vision::Image) -> Array2<f64>,
}

const CANDIDATES: &[Candidate] = &[
    Candidate {
        name: "A baseline (grayscale)",
        normalize: norm_baseline,
    },
    Candidate {
        name: "B contrast stretch",
        normalize: norm_contrast_stretch,
    },
    Candidate {
        name: "C histogram eq",
        normalize: norm_hist_eq,
    },
    Candidate {
        name: "D sobel + max-norm",
        normalize: norm_sobel,
    },
    Candidate {
        name: "E adaptive binary",
        normalize: norm_adaptive_binary,
    },
];

// ---------------------------------------------------------------------
// card-level test pair extraction
// ---------------------------------------------------------------------

struct TestPair {
    #[allow(dead_code)]
    id: String,
    colored: vision::Image,
    sketch: vision::Image,
}

fn extract_test_pairs(dump_dir: &PathBuf) -> Vec<TestPair> {
    let refs = &*assets::REFERENCE_PAGES_WIN;
    let mut pairs = Vec::new();
    for &(new_idx, old_idx) in CONFIRMED_MAPPING {
        let path = dump_dir.join(format!("{:02}.png", new_idx));
        let new_page = match pipeline::imread(&path) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("skip {}: {}", path.display(), e);
                continue;
            }
        };
        let old_page = &refs[old_idx];
        let new_cards = vision::crop_cards(&new_page, &WIN_HD);
        let old_cards = vision::crop_cards(old_page, &WIN_HD);
        for (slot, (nc, oc)) in new_cards.iter().zip(old_cards.iter()).enumerate() {
            let new_state = classify(nc);
            let old_state = classify(oc);
            if new_state == SlotState::Colored && old_state == SlotState::Sketch {
                pairs.push(TestPair {
                    id: format!("new{:02}_old{:02}_slot{:02}", new_idx, old_idx, slot),
                    colored: nc.clone(),
                    sketch: oc.clone(),
                });
            }
        }
    }
    pairs
}

// ---------------------------------------------------------------------
// card-level benchmark
// ---------------------------------------------------------------------

fn run_card_level(pairs: &[TestPair]) {
    println!(
        "\n=== Card-level benchmark: {} (colored, own-sketch) test pairs ===",
        pairs.len()
    );
    println!(
        "{:<28} {:>10} {:>12} {:>14}",
        "candidate", "top1_acc", "mean_margin", "mean_mse_own"
    );
    for cand in CANDIDATES {
        let colored_norm: Vec<Array2<f64>> =
            pairs.iter().map(|p| (cand.normalize)(&p.colored)).collect();
        let sketch_norm: Vec<Array2<f64>> =
            pairs.iter().map(|p| (cand.normalize)(&p.sketch)).collect();

        let mut correct = 0usize;
        let mut margins = Vec::new();
        for (i, c) in colored_norm.iter().enumerate() {
            let mut sims: Vec<(usize, f64)> = sketch_norm
                .iter()
                .enumerate()
                .map(|(j, s)| (j, ncc_arr(c, s)))
                .collect();
            sims.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            let own_sim = sims.iter().find(|(j, _)| *j == i).unwrap().1;
            let best_other = sims.iter().find(|(j, _)| *j != i).unwrap().1;
            if sims[0].0 == i {
                correct += 1;
            }
            margins.push(own_sim - best_other);
        }
        let acc = correct as f64 / pairs.len() as f64;
        let mean_margin = margins.iter().sum::<f64>() / margins.len() as f64;

        // plain MSE for reference (no normalization other than grayscale)
        let mean_mse_own: f64 = pairs
            .iter()
            .map(|p| vision::mse(&p.colored, &p.sketch) as f64)
            .sum::<f64>()
            / pairs.len() as f64;

        println!(
            "{:<28} {:>9.1}% {:>12.4} {:>14.1}",
            cand.name,
            acc * 100.0,
            mean_margin,
            mean_mse_own
        );
    }
}

// ---------------------------------------------------------------------
// page-level benchmark: redo match_debug with normalization applied
// ---------------------------------------------------------------------

fn run_page_level(dump_dir: &PathBuf) {
    println!("\n=== Page-level benchmark: 26 screenshots vs 22 old refs (NCC margin, confirmed pages only) ===");
    let refs = &*assets::REFERENCE_PAGES_WIN;
    let mut paths: Vec<PathBuf> = std::fs::read_dir(dump_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|e| e == "png").unwrap_or(false))
        .collect();
    paths.sort();

    println!(
        "{:<28} {:>12} {:>12}",
        "candidate", "avg_margin", "min_margin"
    );
    for cand in CANDIDATES {
        let ref_norm: Vec<Array2<f64>> = refs.iter().map(cand.normalize).collect();
        let mut margins = Vec::new();
        for (new_idx, _) in CONFIRMED_MAPPING {
            let path = &paths[*new_idx];
            let page = match pipeline::imread(path) {
                Ok(p) => p,
                Err(_) => continue,
            };
            let page_norm = (cand.normalize)(&page);
            let mut sims: Vec<(usize, f64)> = ref_norm
                .iter()
                .enumerate()
                .map(|(i, r)| (i, ncc_arr(&page_norm, r)))
                .collect();
            sims.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            let margin = sims[0].1 - sims[1].1;
            margins.push(margin);
        }
        let avg = margins.iter().sum::<f64>() / margins.len() as f64;
        let min = margins.iter().cloned().fold(f64::INFINITY, f64::min);
        println!("{:<28} {:>12.4} {:>12.4}", cand.name, avg, min);
    }
}

fn main() {
    let dump_dir: PathBuf = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "../data/processed/reference_new".to_string())
        .into();

    let pairs = extract_test_pairs(&dump_dir);
    run_card_level(&pairs);
    run_page_level(&dump_dir);
}
