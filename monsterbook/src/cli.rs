extern crate clap;

use clap::{AppSettings, Parser, Subcommand};
use monsterbook::{crop, stitch, utils};
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
        Commands::ReferenceBook { source, output } => {
            fs::create_dir_all(output)?;
            let images = utils::get_cropped_images(source)?;
            let names = utils::page_metadata().into_iter().map(|metadata| {
                let mut output = output.clone();
                output.push(format!(
                    "{:02}_{}_{}.png",
                    metadata.page_id, metadata.tab_color, metadata.tab_index
                ));
                output
            });
            for (img, name) in images.iter().zip(names) {
                crop::imsave(&name, &img)?;
            }
        }
        Commands::StitchPages { source, output } => {
            let images = utils::get_cropped_images(source)?;
            let stitched = stitch::stitch_images(images, 6);
            crop::imsave(&output, &stitched)?;
        }
        Commands::StitchCards {
            source,
            output,
            generate_stats,
        } => {
            let mut images = utils::get_cropped_images(source)?;
            if *generate_stats {
                return Ok(println!("{:?}", utils::get_empty_card_mse(&mut images)));
            }
            let stitched = utils::stitch_cards(&mut images);
            crop::imsave(&output, &stitched)?;
        }
    }
    Ok(())
}
