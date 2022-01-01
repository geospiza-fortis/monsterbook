extern crate clap;

use clap::{AppSettings, Parser, Subcommand};
use monsterbook::crop;
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
            let img = crop::imread(source)?;
            let (x, y) = crop::match_reference_page(&img)?;
            let cropped = crop::crop(img, x, y)?;
            crop::imsave(output, cropped)?;
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
                let img = crop::imread(&entry.path())?;
                // we only run match reference on the first iteration, we also
                // assume that it's impossible to have an image where the offset
                // is at 0, 0
                if x == 0 && y == 0 {
                    let (a, b) = crop::match_reference_page(&img)?;
                    x = a;
                    y = b;
                }
                let cropped = crop::crop(img, x, y)?;

                let name = format!(
                    "{:02}_{}_{}.png",
                    metadata.page_id, metadata.tab_color, metadata.tab_index
                );
                let mut output = output.clone();
                output.push(name);
                crop::imsave(&output, cropped)?;
            }
        }
    }
    Ok(())
}
