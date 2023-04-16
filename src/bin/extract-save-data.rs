use anyhow::Result;
use clap::Parser;
use select_save::extractor::Extractor;
use std::path::PathBuf;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long)]
    file: PathBuf,

    #[arg(long)]
    config: PathBuf,

    #[clap(flatten)]
    verbose: clap_verbosity_flag::Verbosity,
}

fn main() -> Result<()> {
    let args = Args::parse();
    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();

    let extractor = Extractor::new(&args.config)?;

    for result in extractor.extract(&args.file)? {
        println!("{result}")
    }

    Ok(())
}
