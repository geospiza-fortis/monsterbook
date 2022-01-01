extern crate clap;

use clap::{AppSettings, Parser, Subcommand};
use monsterbook::{crop, stitch};
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[clap(name = "monsterbook")]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Crop a single screenshot
    #[clap(setting(AppSettings::ArgRequiredElseHelp))]
    Crop {
        #[clap(required = true, parse(from_os_str))]
        source: PathBuf,
        #[clap(required = true, parse(from_os_str))]
        output: PathBuf,
    },
    /// Crop cards from a single screenshot
    #[clap(setting(AppSettings::ArgRequiredElseHelp))]
    CropCards {
        #[clap(required = true, parse(from_os_str))]
        source: PathBuf,
        #[clap(required = true, parse(from_os_str))]
        output: PathBuf,
    },
    /// Generate the reference pages with the appropriate filenames
    #[clap(setting(AppSettings::ArgRequiredElseHelp))]
    ReferenceBook {
        #[clap(required = true, parse(from_os_str))]
        source: PathBuf,
        #[clap(required = true, parse(from_os_str))]
        output: PathBuf,
        #[clap(long="no-overwrite", parse(from_flag = std::ops::Not::not))]
        overwrite: bool,
    },
    /// Create a stitched image of full pages
    #[clap(setting(AppSettings::ArgRequiredElseHelp))]
    StitchPages {
        #[clap(required = true, parse(from_os_str))]
        source: PathBuf,
        #[clap(required = true, parse(from_os_str))]
        output: PathBuf,
    },
    /// Create a stitched image of cards
    #[clap(setting(AppSettings::ArgRequiredElseHelp))]
    StitchCards {
        #[clap(required = true, parse(from_os_str))]
        source: PathBuf,
        #[clap(required = true, parse(from_os_str))]
        output: PathBuf,
        #[clap(long = "generate-stats", parse(from_flag))]
        generate_stats: bool,
    },
}

struct PageMetadata {
    page_id: u8,
    tab_color: String,
    tab_index: u8,
}

fn page_metadata() -> Vec<PageMetadata> {
    const TAB_COUNTS: [(&str, u8); 9] = [
        ("red", 1),
        ("orange", 3),
        ("lightgreen", 4),
        ("green", 3),
        ("lightblue", 3),
        ("blue", 2),
        ("purple", 2),
        ("black", 2),
        ("gold", 3),
    ];
    let mut meta = Vec::new();
    let mut page_id = 0;
    for (color, count) in TAB_COUNTS {
        for i in 0..count {
            meta.push(PageMetadata {
                page_id: page_id,
                tab_color: color.into(),
                tab_index: i,
            });
            page_id += 1;
        }
    }
    return meta;
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    match &args.command {
        Commands::Crop { source, output } => {
            // it's totally possible that the image is poorly formatted, so we
            // guess the type
            let mut img = crop::imread(source)?;
            let (x, y) = crop::match_reference_page(&img)?;
            let cropped = crop::crop(&mut img, x, y)?;
            crop::imsave(output, &cropped)?;
        }
        Commands::CropCards { source, output } => {
            // it's totally possible that the image is poorly formatted, so we
            // guess the type
            let mut img = crop::imread(source)?;
            let (x, y) = crop::match_reference_page(&img)?;
            let mut cropped = crop::crop(&mut img, x, y)?;
            // now lets crop, remove all the empty entries
            fs::create_dir_all(output)?;
            let cards = crop::crop_cards(&mut cropped)?;
            for (i, card) in cards.iter().enumerate() {
                let mut card_file = output.clone();
                card_file.push(format!("{:02}.png", i));
                crop::imsave(&card_file, card)?;
            }
        }
        Commands::ReferenceBook {
            source,
            output,
            overwrite,
        } => {
            if output.exists() && !overwrite {
                // TODO: better error handling...
                panic!("path already exists");
            }
            fs::create_dir_all(output)?;
            // implicitly reads this in the correct order
            let (mut x, mut y) = (0, 0);
            for (entry, metadata) in fs::read_dir(source)?.zip(page_metadata().iter()) {
                let entry = entry?;
                let mut img = crop::imread(&entry.path())?;
                // we only run match reference on the first iteration, we also
                // assume that it's impossible to have an image where the offset
                // is at 0, 0
                if x == 0 && y == 0 {
                    let (a, b) = crop::match_reference_page(&img)?;
                    x = a;
                    y = b;
                }
                let cropped = crop::crop(&mut img, x, y)?;

                let name = format!(
                    "{:02}_{}_{}.png",
                    metadata.page_id, metadata.tab_color, metadata.tab_index
                );
                let mut output = output.clone();
                output.push(name);
                crop::imsave(&output, &cropped)?;
            }
        }
        Commands::StitchPages { source, output } => {
            let mut images = Vec::new();
            // output is a file
            let (mut x, mut y) = (0, 0);
            for entry in fs::read_dir(source)? {
                let mut img = crop::imread(&entry?.path())?;
                if x == 0 && y == 0 {
                    let (a, b) = crop::match_reference_page(&img)?;
                    x = a;
                    y = b;
                }
                let cropped = crop::crop(&mut img, x, y)?;
                images.push(cropped);
            }
            let stitched = stitch::stitch_images(images, 6);
            crop::imsave(&output, &stitched)?;
        }
        Commands::StitchCards {
            source,
            output,
            generate_stats,
        } => {
            let mut images = Vec::new();
            // output is a file
            let (mut x, mut y) = (0, 0);
            for entry in fs::read_dir(source)? {
                let mut img = crop::imread(&entry?.path())?;
                if x == 0 && y == 0 {
                    let (a, b) = crop::match_reference_page(&img)?;
                    x = a;
                    y = b;
                }
                let cropped = crop::crop(&mut img, x, y)?;
                images.push(cropped);
            }
            // now lets crop, remove all the empty entries
            let cards_iter = images
                .iter_mut()
                .flat_map(|img| crop::crop_cards(img).unwrap());

            // instead of filtering cards, we'll print statistics out to stdout,
            // which can be used to determine what the threshold should be
            if *generate_stats {
                let mse: Vec<u32> = cards_iter.map(|img| crop::card_mse(&img)).collect();
                println!("{:?}", mse);
                // exit early...
                return Ok(());
            }
            // to determine the threshold, generate stats and look for an obvious cutoff
            let cards = cards_iter.filter(|img| crop::card_mse(img) > 500).collect();
            let stitched = stitch::stitch_images(cards, 6 * 5);
            crop::imsave(&output, &stitched)?;
        }
    }
    Ok(())
}
