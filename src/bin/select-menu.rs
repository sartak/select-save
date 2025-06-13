use anyhow::Result;
use clap::Parser;
use gag::Gag;
use select_save::{manager, scene::selectstring, ui};
use std::io::{self, BufRead};
use std::path::PathBuf;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, env = "SCREEN_WIDTH")]
    width: u32,

    #[arg(long, env = "SCREEN_HEIGHT")]
    height: u32,

    #[arg(
        long,
        default_value = "/usr/share/fonts/truetype/noto/NotoMono-Regular.ttf"
    )]
    font: PathBuf,

    #[arg(long, default_value = "Make a selection")]
    title: String,
}

fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let Args {
        height,
        width,
        font,
        title,
    } = Args::parse();

    info!("Launching SDL {width}x{height}");

    // Read items from stdin
    let items: Vec<String> = io::stdin().lock().lines().collect::<Result<_, _>>()?;

    if items.is_empty() {
        return Ok(());
    }

    // Gag stdout to suppress driver output
    let gag = Gag::stdout()?;

    let root_scene = Box::new(selectstring::SelectString::new(items, title));
    let manager = manager::Manager::new(root_scene);
    match ui::run(width, height, &font, manager)? {
        Some(selectstring::Operation::SelectItem(item)) => {
            // Temporarily ungag to print result
            drop(gag);
            println!("{}", item);
        }
        None => {}
    }

    // Gag again
    let _gag = Gag::stdout()?;

    info!("Shutting down");

    Ok(())
}
