extern crate clap;

use clap::{Parser, Subcommand};
use monsterbook::layout::WIN_HD;
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
            let stitched = pipeline::stitch_cards(&images, 4 * 6, &WIN_HD);
            println!("stitched cards");
            pipeline::imsave(output, &stitched)?;
        }
    }
    Ok(())
}
