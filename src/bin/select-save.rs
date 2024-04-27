use anyhow::Result;
use clap::Parser;
use select_save::{manager, ui};
use std::path::PathBuf;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
struct Args {
    #[arg(long)]
    width: u32,

    #[arg(long)]
    height: u32,

    #[arg(long)]
    root: PathBuf,

    #[arg(long)]
    destination: PathBuf,

    #[arg(long)]
    exec_command: Option<PathBuf>,

    #[arg(
        long,
        default_value = "/usr/share/fonts/truetype/noto/NotoMono-Regular.ttf"
    )]
    font: PathBuf,
}

fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args = Args::parse();

    info!("Launching SDL {}x{}", args.width, args.height);

    let manager = manager::Manager::new(args.root, args.destination, args.exec_command);
    ui::run(args.width, args.height, &args.font, manager)?;

    info!("Shutting down");

    Ok(())
}
