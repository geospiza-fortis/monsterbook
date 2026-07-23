//! Investigate whether the tab-ribbon strip to the left of the cropped page
//! carries a usable per-page identification signal (its highlighted tab).
use monsterbook::layout::WIN_HD;
use monsterbook::pipeline;
use std::path::PathBuf;

fn main() {
    let source: PathBuf = std::env::args().nth(1).unwrap().into();
    let output: PathBuf = std::env::args().nth(2).unwrap().into();
    std::fs::create_dir_all(&output).unwrap();

    let mut paths: Vec<PathBuf> = std::fs::read_dir(&source)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    paths.sort();

    let mut anchor: Option<(u32, u32)> = None;
    for (i, path) in paths.iter().enumerate() {
        let mut img = pipeline::imread(path).unwrap();
        let (x, y) = *anchor.get_or_insert_with(|| pipeline::match_reference_page(&img));
        println!(
            "{}: anchor=({}, {}) img=({}, {})",
            path.display(),
            x,
            y,
            img.width(),
            img.height()
        );
        if x > 0 {
            let strip_w = x.min(60);
            let strip_x = x - strip_w;
            let strip =
                image::imageops::crop(&mut img, strip_x, y, strip_w, WIN_HD.page_height).to_image();
            let mut out = output.clone();
            out.push(format!("{:02}_ribbon.png", i));
            pipeline::imsave(&out, &strip).unwrap();
        }
    }
}
