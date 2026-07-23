extern crate clap;

use clap::{Parser, Subcommand};
use monsterbook::layout::{STITCH_CARDS_PER_ROW, WIN_HD};
use monsterbook::{pipeline, vision};
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
    #[clap(arg_required_else_help = true)]
    Crop {
        #[clap(required = true, parse(from_os_str))]
        source: PathBuf,
        #[clap(required = true, parse(from_os_str))]
        output: PathBuf,
    },
    /// Crop cards from a single screenshot
    #[clap(arg_required_else_help = true)]
    CropCards {
        #[clap(required = true, parse(from_os_str))]
        source: PathBuf,
        #[clap(required = true, parse(from_os_str))]
        output: PathBuf,
    },
    /// Generate the reference pages with the appropriate filenames
    #[clap(arg_required_else_help = true)]
    ReferenceBook {
        #[clap(required = true, parse(from_os_str))]
        source: PathBuf,
        #[clap(required = true, parse(from_os_str))]
        output: PathBuf,
    },
    /// Create a stitched image of full pages
    #[clap(arg_required_else_help = true)]
    StitchPages {
        #[clap(required = true, parse(from_os_str))]
        source: PathBuf,
        #[clap(required = true, parse(from_os_str))]
        output: PathBuf,
    },
    /// Create a stitched image of cards
    #[clap(arg_required_else_help = true)]
    StitchCards {
        #[clap(required = true, parse(from_os_str))]
        source: PathBuf,
        #[clap(required = true, parse(from_os_str))]
        output: PathBuf,
        #[clap(long = "generate-stats", parse(from_flag))]
        generate_stats: bool,
    },
    /// Transcribe screenshots via content-addressed session ingestion:
    /// screenshots may be in any order and pages are identified by content
    #[clap(arg_required_else_help = true)]
    SessionTranscribe {
        #[clap(required = true, parse(from_os_str))]
        source: PathBuf,
        #[clap(required = true, parse(from_os_str))]
        output: PathBuf,
    },
    /// Transcribe screenshots into a JSON summary of entries and counts
    #[clap(arg_required_else_help = true)]
    Transcribe {
        #[clap(required = true, parse(from_os_str))]
        source: PathBuf,
        #[clap(required = true, parse(from_os_str))]
        output: PathBuf,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    match &args.command {
        Commands::Crop { source, output } => {
            // it's totally possible that the image is poorly formatted, so we
            // guess the type
            let mut img = pipeline::imread(source)?;
            let (x, y) = pipeline::match_reference_page(&img);
            let cropped = vision::crop_page(&mut img, x, y, &WIN_HD);
            pipeline::imsave(output, &cropped)?;
        }
        Commands::CropCards { source, output } => {
            // it's totally possible that the image is poorly formatted, so we
            // guess the type
            let mut img = pipeline::imread(source)?;
            let (x, y) = pipeline::match_reference_page(&img);
            let cropped = vision::crop_page(&mut img, x, y, &WIN_HD);
            fs::create_dir_all(output)?;
            let cards = vision::crop_cards(&cropped, &WIN_HD);
            for (i, card) in cards.iter().enumerate() {
                let mut card_file = output.clone();
                card_file.push(format!("{:02}.png", i));
                pipeline::imsave(&card_file, card)?;
            }
        }
        Commands::ReferenceBook { source, output } => {
            fs::create_dir_all(output)?;
            let images = pipeline::load_pages(source, &WIN_HD)?;
            let names = pipeline::reference_page_names().into_iter().map(|name| {
                let mut output = output.clone();
                output.push(name);
                output
            });
            for (img, name) in images.iter().zip(names) {
                pipeline::imsave(&name, img)?;
            }
        }
        Commands::StitchPages { source, output } => {
            let images = pipeline::load_pages(source, &WIN_HD)?;
            let stitched = vision::stitch_images(images, 6);
            pipeline::imsave(output, &stitched)?;
        }
        Commands::StitchCards {
            source,
            output,
            generate_stats,
        } => {
            let images = pipeline::load_pages(source, &WIN_HD)?;
            if *generate_stats {
                return Ok(println!("{:?}", pipeline::empty_card_mse(&images, &WIN_HD)));
            }
            let stitched = pipeline::stitch_cards(&images, STITCH_CARDS_PER_ROW, &WIN_HD);
            println!("stitched cards");
            pipeline::imsave(output, &stitched)?;
        }
        Commands::SessionTranscribe { source, output } => {
            let mut session = monsterbook::session::Session::new();
            let mut paths: Vec<PathBuf> = fs::read_dir(source)?
                .map(|entry| entry.map(|e| e.path()))
                .collect::<Result<_, _>>()?;
            paths.sort();
            for path in paths {
                let bytes = fs::read(&path)?;
                match session.add_screenshot(&bytes) {
                    Ok(page_id) => println!("{}: page {}", path.display(), page_id),
                    Err(err) => eprintln!("{}: skipped ({})", path.display(), err),
                }
            }
            let missing = session.missing();
            if !missing.is_empty() {
                eprintln!("missing pages: {:?}", missing);
            }
            let entries = session.transcribe();
            let doc = serde_json::json!({ "data": entries });
            if let Some(parent) = output.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(output, serde_json::to_string_pretty(&doc)?)?;
        }
        Commands::Transcribe { source, output } => {
            let images = pipeline::load_pages(source, &WIN_HD)?;
            let entries = pipeline::transcribe(&images, &monsterbook::assets::BOOK, &WIN_HD);
            let doc = serde_json::json!({ "data": entries });
            fs::write(output, serde_json::to_string_pretty(&doc)?)?;
        }
    }
    Ok(())
}
