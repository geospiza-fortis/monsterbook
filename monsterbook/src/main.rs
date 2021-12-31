extern crate clap;

use clap::{AppSettings, Parser, Subcommand};
use monsterbook::crop;
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    match &args.command {
        Commands::Crop { source, output } => {
            // it's totally possible that the image is poorly formatted, so we
            // guess the type
            let img = crop::imread(source)?;
            let cropped = crop::crop(img);
            crop::imsave(output, cropped)?;
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
