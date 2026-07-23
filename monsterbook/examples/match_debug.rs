use monsterbook::{assets, vision};
use std::path::Path;

fn main() {
    let dir = std::env::args().nth(1).expect("usage: match_debug <dir>");
    let mut paths: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    paths.sort();
    let refs = &*assets::REFERENCE_PAGES_WIN;
    let mut raw_ratios = Vec::new();
    let mut edge_ratios = Vec::new();
    let mut ncc_ratios = Vec::new();
    for path in paths {
        let mut img = match monsterbook::pipeline::imread(&path) {
            Ok(img) => img,
            Err(_) => continue,
        };
        let (x, y) = vision::match_reference_page(&img, &assets::REFERENCE_PAGE);
        let page = vision::crop_page(&mut img, x, y, &monsterbook::layout::WIN_HD);

        let mut raw: Vec<(usize, u32)> = refs
            .iter()
            .enumerate()
            .map(|(i, r)| (i, vision::mse(&page, r)))
            .collect();
        raw.sort_by_key(|&(_, m)| m);

        let mut edge: Vec<(usize, u32)> = refs
            .iter()
            .enumerate()
            .map(|(i, r)| (i, vision::mse_edges(&page, r)))
            .collect();
        edge.sort_by_key(|&(_, m)| m);

        let mut ncc: Vec<(usize, f64)> = refs
            .iter()
            .enumerate()
            .map(|(i, r)| (i, vision::ncc(&page, r)))
            .collect();
        ncc.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let raw_ratio = raw[1].1 as f64 / (raw[0].1 as f64 + 1.0);
        let edge_ratio = edge[1].1 as f64 / (edge[0].1 as f64 + 1.0);
        // for ncc, "margin" is best-second (both in [-1,1], higher=better)
        let ncc_margin = ncc[0].1 - ncc[1].1;
        raw_ratios.push(raw_ratio);
        edge_ratios.push(edge_ratio);
        ncc_ratios.push(ncc_margin);

        println!(
            "{}: raw best=page{} mse={} second=page{} mse={} ratio={:.2} | edge best=page{} mse={} second=page{} mse={} ratio={:.2} | ncc best=page{} r={:.4} second=page{} r={:.4} margin={:.4}",
            Path::new(&path).file_name().unwrap().to_string_lossy(),
            raw[0].0, raw[0].1, raw[1].0, raw[1].1, raw_ratio,
            edge[0].0, edge[0].1, edge[1].0, edge[1].1, edge_ratio,
            ncc[0].0, ncc[0].1, ncc[1].0, ncc[1].1, ncc_margin,
        );
    }
    let avg = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
    let min = |v: &[f64]| v.iter().cloned().fold(f64::INFINITY, f64::min);
    println!(
        "\nraw: avg_margin_ratio={:.2} min_margin_ratio={:.2}",
        avg(&raw_ratios),
        min(&raw_ratios)
    );
    println!(
        "edge: avg_margin_ratio={:.2} min_margin_ratio={:.2}",
        avg(&edge_ratios),
        min(&edge_ratios)
    );
    println!(
        "ncc: avg_margin={:.4} min_margin={:.4}",
        avg(&ncc_ratios),
        min(&ncc_ratios)
    );
}
