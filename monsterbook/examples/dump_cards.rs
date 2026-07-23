//! Dump specific card crops from a directory of cropped pages, for visual
//! inspection during calibration. Args: <pages_dir> <out_dir> <page:index>...
use monsterbook::layout::WIN_HD;
use monsterbook::{pipeline, vision};
use std::path::PathBuf;

fn main() {
    let mut args = std::env::args().skip(1);
    let source: PathBuf = args.next().expect("usage: dump_cards <pages> <out> <page:index>...").into();
    let output: PathBuf = args.next().expect("usage: dump_cards <pages> <out> <page:index>...").into();
    std::fs::create_dir_all(&output).unwrap();
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&source)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    paths.sort();
    let pages: Vec<_> = paths
        .iter()
        .filter_map(|p| pipeline::imread(p).ok())
        .collect();
    for spec in args {
        let (p, i) = spec.split_once(':').expect("spec must be page:index");
        let (p, i): (usize, usize) = (p.parse().unwrap(), i.parse().unwrap());
        let card = vision::crop_cards(&pages[p], &WIN_HD).swap_remove(i);
        let mut out = output.clone();
        out.push(format!("{:02}_{:02}.png", p, i));
        pipeline::imsave(&out, &card).unwrap();
    }
}
