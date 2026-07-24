//! Ingest a directory of screenshots into a Session and write the stitched
//! card image. Usage: session_stitch <dir> <out.png>
use monsterbook::layout::STITCH_CARDS_PER_ROW;
use monsterbook::session::Session;

fn main() {
    let dir = std::env::args().nth(1).expect("usage: session_stitch <dir> <out.png>");
    let out = std::env::args().nth(2).expect("usage: session_stitch <dir> <out.png>");
    let mut paths: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().map_or(false, |e| e == "png"))
        .collect();
    paths.sort();

    let mut session = Session::new();
    for path in &paths {
        let name = path.file_name().unwrap().to_string_lossy();
        match session.add_screenshot(&std::fs::read(path).unwrap()) {
            Ok(page_id) => println!("{name}: page {page_id}"),
            Err(err) => println!("{name}: ERR {err}"),
        }
    }
    println!("ingested pages: {:?}", session.pages().keys().collect::<Vec<_>>());
    println!("missing pages: {:?}", session.missing());
    let entries = session.transcribe();
    let total: u32 = entries.iter().map(|e| e.count).sum();
    println!("entries: {} total_count: {}", entries.len(), total);
    let stitched = session.stitch(STITCH_CARDS_PER_ROW, false).expect("nothing to stitch");
    monsterbook::pipeline::imsave(std::path::Path::new(&out), &stitched).unwrap();
    println!("wrote {out}");
}
