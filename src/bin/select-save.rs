use anyhow::Result;
use clap::Parser;
use log::info;
use select_save::{manager, ui};
use std::path::PathBuf;

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

    #[clap(flatten)]
    verbose: clap_verbosity_flag::Verbosity,
}

fn main() -> Result<()> {
    let args = Args::parse();
    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();

    info!("Launching SDL {}x{}", args.width, args.height);

    let manager = manager::Manager::new(args.root, args.destination, args.exec_command);
    ui::run(args.width, args.height, &args.font, manager)?;

    info!("Shutting down");

    Ok(())
}
