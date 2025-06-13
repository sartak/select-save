use anyhow::Result;
use clap::Parser;
use select_save::{
    manager,
    scene::selectgame::{Operation, SelectGame},
    ui,
};
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, env = "SCREEN_WIDTH")]
    width: u32,

    #[arg(long, env = "SCREEN_HEIGHT")]
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

    let Args {
        root,
        destination,
        exec_command,
        height,
        width,
        font,
    } = Args::parse();

    info!("Launching SDL {width}x{height}");

    let root_scene = Box::new(SelectGame::new(root, destination));
    let manager = manager::Manager::new(root_scene);
    match ui::run(width, height, &font, manager)? {
        Some(Operation::ExecGame(game)) => {
            if let Some(command) = exec_command {
                let err = Command::new(&command).arg(game).exec();
                error!("Error exec'ing {:?}: {err}", command);
            }
        }
        None => {}
    };

    info!("Shutting down");

    Ok(())
}
