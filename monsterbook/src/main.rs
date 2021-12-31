extern crate clap;

use clap::{AppSettings, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[clap(name = "monsterbook")]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
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

fn main() {
    let args = Cli::parse();
    match &args.command {
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
}
