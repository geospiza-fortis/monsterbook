//! Print identification margins for the embedded reference set: inter-page
//! separation, non-book rejection (solid fills, noise), and cross-set NCC
//! against a directory of real cropped pages.
use image::{Rgba, RgbaImage};
use monsterbook::{assets, pipeline, vision};
use std::path::PathBuf;

fn main() {
    let refs = &assets::REFERENCE_PAGES_WIN;
    let mut worst_ncc: f64 = -1.0;
    let mut worst_mse = u32::MAX;
    for (i, r) in refs.iter().enumerate() {
        let (ncc, mse) = refs
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != i)
            .map(|(_, o)| (vision::ncc(r, o), vision::mse(r, o)))
            .fold((-1.0f64, u32::MAX), |(a, b), (n, m)| (a.max(n), b.min(m)));
        worst_ncc = worst_ncc.max(ncc);
        worst_mse = worst_mse.min(mse);
    }
    println!("inter-ref: max non-self ncc {:.3}, min mse {}", worst_ncc, worst_mse);
    for v in [0u8, 64, 128, 192, 255] {
        let solid = RgbaImage::from_pixel(165, 225, Rgba([v, v, v, 255]));
        let (_, mse) = pipeline::match_min_mse(&solid, refs);
        let (_, ncc) = pipeline::match_max_ncc(&solid, refs);
        println!("solid {:3}: best mse {}, best ncc {:.3}", v, mse, ncc);
    }
    let mut state: u32 = 12345;
    let noise = RgbaImage::from_fn(165, 225, |_, _| {
        state = state.wrapping_mul(1103515245).wrapping_add(12345);
        Rgba([((state >> 16) & 0xff) as u8; 4])
    });
    let (_, mse) = pipeline::match_min_mse(&noise, refs);
    let (_, ncc) = pipeline::match_max_ncc(&noise, refs);
    println!("noise: best mse {}, best ncc {:.3}", mse, ncc);

    if let Some(dir) = std::env::args().nth(1) {
        let dir: PathBuf = dir.into();
        let mut paths: Vec<PathBuf> = std::fs::read_dir(&dir).unwrap().map(|e| e.unwrap().path()).collect();
        paths.sort();
        let mut correct = 0;
        let mut min_ncc: f64 = 2.0;
        let mut total = 0;
        for (i, p) in paths.iter().enumerate() {
            let img = match pipeline::imread(p) { Ok(i) => i, Err(_) => continue };
            let (index, ncc) = pipeline::match_max_ncc(&img, refs);
            if index == i { correct += 1; min_ncc = min_ncc.min(ncc); }
            else { println!("{:02} -> {:02} ncc={:.3} WRONG", i, index, ncc); }
            total += 1;
        }
        println!("real pages: {}/{} correct, min correct ncc {:.3}", correct, total, min_ncc);
        let mut mse_correct = 0;
        for (i, p) in paths.iter().enumerate() {
            let img = match pipeline::imread(p) { Ok(i) => i, Err(_) => continue };
            let (index, _) = pipeline::match_min_mse(&img, refs);
            if index == i { mse_correct += 1; } else { println!("mse {:02} -> {:02} WRONG", i, index); }
        }
        println!("real pages by mse: {}/{} correct", mse_correct, total);
    }
}
