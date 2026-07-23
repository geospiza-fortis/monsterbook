//! Crop every screenshot in a directory (win-scale, native) and save as
//! {index:02}.png in an output directory, in filename-sorted order. Used to
//! build a scratch reference set from a freshly collected book, independent
//! of the embedded (stale, 22-page) book.json naming.
use monsterbook::layout::WIN_HD;
use monsterbook::{pipeline, vision};
use std::path::PathBuf;

fn main() {
    let mut args = std::env::args().skip(1);
    let source: PathBuf = args.next().expect("usage: dump_pages <source> <output>").into();
    let output: PathBuf = args.next().expect("usage: dump_pages <source> <output>").into();
    std::fs::create_dir_all(&output).unwrap();

    let mut paths: Vec<PathBuf> = std::fs::read_dir(&source)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    paths.sort();

    let mut anchor: Option<(u32, u32)> = None;
    for (i, path) in paths.iter().enumerate() {
        let mut img = match pipeline::imread(path) {
            Ok(img) => img,
            Err(_) => continue,
        };
        let (x, y) = match anchor {
            Some(a) => a,
            None => {
                let a = pipeline::match_reference_page(&img);
                anchor = Some(a);
                a
            }
        };
        let page = vision::crop_page(&mut img, x, y, &WIN_HD);
        let mut out = output.clone();
        out.push(format!("{:02}.png", i));
        pipeline::imsave(&out, &page).unwrap();
        println!("{}: {:02}.png ({}x{})", path.display(), i, page.width(), page.height());
    }
}
