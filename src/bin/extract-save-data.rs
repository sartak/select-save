use anyhow::Result;
use clap::Parser;
use select_save::extractor::Extractor;
use std::path::PathBuf;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
struct Args {
    #[arg(long)]
    file: PathBuf,

    #[arg(long)]
    config: PathBuf,
}

fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args = Args::parse();

    let extractor = Extractor::new(&args.config)?;

    for result in extractor.extract(&args.file)? {
        println!("{result}")
    }

    Ok(())
}
