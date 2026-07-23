//! Regenerate the empty-card template from a directory of cropped empty
//! pages, and print the MSE distributions needed to pick data-driven
//! empty/unseen thresholds: empty cards vs the new template, and a populated
//! reference set's cards vs the same template.
use image::RgbaImage;
use monsterbook::layout::WIN_HD;
use monsterbook::{pipeline, vision};
use std::path::{Path, PathBuf};

fn load_cropped_pages(dir: &Path) -> Vec<RgbaImage> {
    let mut paths: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    paths.sort();
    paths
        .iter()
        .filter_map(|p| pipeline::imread(p).ok())
        .collect()
}

fn mean_card(cards: &[RgbaImage]) -> RgbaImage {
    let (w, h) = (cards[0].width(), cards[0].height());
    let mut acc = vec![0u64; (w * h * 4) as usize];
    for card in cards {
        for (a, p) in acc.iter_mut().zip(card.as_raw()) {
            *a += *p as u64;
        }
    }
    let n = cards.len() as u64;
    let raw: Vec<u8> = acc.iter().map(|a| ((a + n / 2) / n) as u8).collect();
    RgbaImage::from_raw(w, h, raw).unwrap()
}

fn percentile(sorted: &[u32], p: f64) -> u32 {
    sorted[((sorted.len() - 1) as f64 * p).round() as usize]
}

fn summarize(label: &str, mses: &mut Vec<u32>) {
    mses.sort_unstable();
    println!(
        "{}: n={} min={} p50={} p95={} p99={} max={}",
        label,
        mses.len(),
        mses[0],
        percentile(mses, 0.50),
        percentile(mses, 0.95),
        percentile(mses, 0.99),
        mses.last().unwrap()
    );
}

fn main() {
    let mut args = std::env::args().skip(1);
    let usage = "usage: calibrate_empty <empty_pages> <populated_pages> <template_out>";
    let empty_dir: PathBuf = args.next().expect(usage).into();
    let populated_dir: PathBuf = args.next().expect(usage).into();
    let template_out: PathBuf = args.next().expect(usage).into();

    let empty_pages = load_cropped_pages(&empty_dir);
    let empty_cards: Vec<RgbaImage> = empty_pages
        .iter()
        .flat_map(|p| vision::crop_cards(p, &WIN_HD))
        .collect();
    let template = mean_card(&empty_cards);
    pipeline::imsave(&template_out, &template).unwrap();
    println!(
        "template: {}x{} from {} cards -> {}",
        template.width(),
        template.height(),
        empty_cards.len(),
        template_out.display()
    );

    let mut empty_mses: Vec<u32> = empty_cards
        .iter()
        .map(|c| vision::mse(c, &template))
        .collect();
    summarize("empty vs new template", &mut empty_mses);

    let mut old_mses: Vec<u32> = empty_cards
        .iter()
        .map(|c| pipeline::card_mse(c))
        .collect();
    summarize("empty vs old template", &mut old_mses);

    // Populated pages contain a mix of empty, unseen, and seen cards; the
    // per-card MSEs against the template should cluster accordingly.
    let populated_pages = load_cropped_pages(&populated_dir);
    let mut labeled: Vec<(u32, usize, usize)> = populated_pages
        .iter()
        .enumerate()
        .flat_map(|(page_id, page)| {
            vision::crop_cards(page, &WIN_HD)
                .into_iter()
                .enumerate()
                .map(move |(index, card)| (page_id, index, card))
        })
        .map(|(page_id, index, card)| (vision::mse(&card, &template), page_id, index))
        .collect();
    let mut populated_mses: Vec<u32> = labeled.iter().map(|(m, _, _)| *m).collect();
    summarize("populated vs new template", &mut populated_mses);

    let mut populated_old: Vec<u32> = populated_pages
        .iter()
        .flat_map(|p| vision::crop_cards(p, &WIN_HD))
        .map(|c| pipeline::card_mse(&c))
        .collect();
    summarize("populated vs old template", &mut populated_old);
    let seen_old: Vec<u32> = populated_old.iter().copied().filter(|m| *m > 3000).collect();
    println!("populated vs old template, first values above 3000: {:?}", &seen_old[..8.min(seen_old.len())]);

    labeled.sort_unstable();
    println!("populated per-card MSE (sorted, mse page:index):");
    for (mse, page_id, index) in &labeled {
        println!("{} {:02}:{:02}", mse, page_id, index);
    }
}
