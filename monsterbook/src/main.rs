extern crate clap;
extern crate image;
extern crate imageproc;

use clap::{AppSettings, Parser, Subcommand};
use image::{imageops, io};
use std::path::PathBuf;

#[derive(Parser)]
#[clap(name = "monsterbook")]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[clap(setting(AppSettings::ArgRequiredElseHelp))]
    Crop {
        #[clap(required = true, parse(from_os_str))]
        source: PathBuf,
        #[clap(required = true, parse(from_os_str))]
        output: PathBuf,
    },
    /// Generate the reference
    #[clap(setting(AppSettings::ArgRequiredElseHelp))]
    ReferenceBook {
        #[clap(required = true, parse(from_os_str))]
        source: PathBuf,
        #[clap(required = true, parse(from_os_str))]
        output: PathBuf,
        #[clap(long="no-overwrite", parse(from_flag = std::ops::Not::not))]
        overwrite: bool,
    },
}

fn path_as_string(path: &PathBuf) -> String {
    path.clone().into_os_string().into_string().unwrap()
}

fn main() -> Result<(), Box<std::error::Error>> {
    let args = Cli::parse();
    match &args.command {
        Commands::Crop { source, output } => {
            // it's totally possible that the image is poorly formatted...
            let mut img = io::Reader::open(&path_as_string(source))?
                .with_guessed_format()?
                .decode()?;
            let (x, y) = (152, 295);
            let (width, height) = (225, 165);
            let cropped = imageops::crop(&mut img, x, y, width, height).to_image();
            cropped.save(&path_as_string(output))?;
        }
        Commands::ReferenceBook {
            source,
            output,
            overwrite,
        } => {
            println!(
                "source {:?}, output {:?}, overwrite {:?}",
                source, output, overwrite
            )
        }
    }
    Ok(())
}
