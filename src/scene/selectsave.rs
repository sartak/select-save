use super::Scene;
use super::message::Message;
use crate::{
    extractor::Extractor,
    internal::{files_for_directory, full_extension, remove_full_extension},
    manager::Action,
    scene::selectgame::Operation,
    ui::{
        Button,
        list::{List, PADDING, preview_width_for_screen_width},
        screen::{Color, FontSize, Rect, Screen},
    },
};
use anyhow::{Result, anyhow};
use chrono::prelude::*;
use itertools::Itertools;
use rand::Rng;
use regex::Regex;
use std::{
    cmp::min,
    collections::{HashMap, VecDeque},
    path::{Path, PathBuf},
    sync::OnceLock,
};
use tracing::info;

pub(super) struct SelectSave {
    game: PathBuf,
    root: PathBuf,
    destination: PathBuf,
    list: List<(PathBuf, Option<PathBuf>)>,
    offset: usize,
    extractor: Option<Extractor>,
}

fn saves_for_game(game: &Path) -> Vec<(PathBuf, Option<PathBuf>)> {
    let mut images: HashMap<std::ffi::OsString, PathBuf> = HashMap::new();
    let mut saves = Vec::new();

    for file in files_for_directory(game) {
        match file.extension().and_then(|p| p.to_str()) {
            Some("png" | "jpg") => {
                let mut stem = file.clone();
                remove_full_extension(&mut stem);
                images.insert(stem.into_os_string(), file);
            }
            Some(_) => {
                saves.push(file);
            }
            None => {}
        }
    }

    saves
        .into_iter()
        .map(move |p| {
            let mut image = p.clone();
            remove_full_extension(&mut image);
            let image = images.get(&image.into_os_string()).cloned();
            (p, image)
        })
        .collect()
}

impl SelectSave {
    pub(super) fn new(game: PathBuf, root: PathBuf, destination: PathBuf) -> Self {
        let offset = rand::rng().random_range(100..999);
        let saves = saves_for_game(&game);
        let list = List::new(saves, "Select a save".to_string());

        let extractor = match game.strip_prefix(&root) {
            Ok(prefix) => {
                let mut config = destination.clone();
                config.push(prefix);
                remove_full_extension(&mut config);
                config.set_extension("extract");

                config
                    .exists()
                    .then(|| Extractor::new(&config).ok())
                    .flatten()
            }
            Err(_) => None,
        };

        Self {
            game,
            list,
            root,
            destination,
            offset,
            extractor,
        }
    }

    fn label_for(&self, save: &Path) -> String {
        save.strip_prefix(&self.game)
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned()
    }

    fn timestamp_in_filename(&self, file: &Path) -> Option<DateTime<chrono::Local>> {
        let mut name = file.strip_prefix(&self.game).ok()?.to_path_buf();
        remove_full_extension(&mut name);
        let stem = name.to_str()?;

        static RE: OnceLock<Regex> = OnceLock::new();
        let re = RE.get_or_init(|| Regex::new(r"^([0-9][0-9][0-9][0-9])([0-9][0-9])([0-9][0-9])-([0-9][0-9])([0-9][0-9])([0-9][0-9])$").unwrap());

        let caps = re.captures(stem)?;

        let (year, month, day, hour, min, sec) = caps
            .iter()
            .skip(1)
            .map(|m| m.unwrap().as_str())
            .map(|s| s.parse::<u32>().unwrap())
            .collect_tuple()?;

        Local
            .with_ymd_and_hms(year as i32, month, day, hour, min, sec)
            .earliest()
    }

    fn modtime(&self, file: &Path) -> Option<DateTime<chrono::Local>> {
        let modtime = std::fs::metadata(file).ok()?.modified().ok()?;
        let dt: DateTime<Local> = modtime.into();
        Some(dt)
    }

    fn duration_since_save(&self, save: &Path) -> Option<String> {
        let dt;
        let prefix;
        if let Some(d) = self.timestamp_in_filename(save) {
            dt = d;
            prefix = "";
        } else {
            dt = self.modtime(save)?;
            prefix = "Maybe ";
        }

        let now = Local::now();
        let duration = now.signed_duration_since(dt);
        let weeks = duration.num_weeks();
        let days = duration.num_days();
        let hours = duration.num_hours();
        let minutes = duration.num_minutes();
        let years = weeks / 52;

        if years == 1 {
            Some(format!("{prefix}1 year ago"))
        } else if years > 1 {
            Some(format!("{prefix}{years} years ago"))
        } else if weeks == 1 {
            Some(format!("{prefix}1 week ago"))
        } else if weeks > 1 {
            Some(format!("{prefix}{weeks} weeks ago"))
        } else if days == 1 {
            Some(format!("{prefix}1 day ago"))
        } else if days > 1 {
            Some(format!("{prefix}{days} days ago"))
        } else if hours == 1 {
            Some(format!("{prefix}1 hour ago"))
        } else if hours > 1 {
            Some(format!("{prefix}{hours} hours ago"))
        } else if minutes == 1 {
            Some(format!("{prefix}1 minute ago"))
        } else if minutes > 1 {
            Some(format!("{prefix}{minutes} minutes ago"))
        } else {
            Some(format!("{prefix}<1 minute ago"))
        }
    }

    fn extract_save(&self, save: &Path) -> Result<Vec<String>> {
        match &self.extractor {
            Some(e) => e.extract(save),
            None => Ok(Vec::new()),
        }
    }

    fn metadata_for_save(&self, save: &Path) -> Vec<String> {
        let mut metadata = Vec::new();

        if let Some(duration) = self.duration_since_save(save) {
            metadata.push(duration);
        }

        if let Ok(meta) = self.extract_save(save) {
            metadata.extend(meta);
        }

        metadata
    }

    fn current_save(&self) -> &(PathBuf, Option<PathBuf>) {
        self.list.current_item().unwrap()
    }

    fn commit_save(&self) -> Result<Vec<String>> {
        let mut results = VecDeque::new();
        let (current_save, _) = self.current_save();
        let label = self.label_for(current_save);

        let mut destination = self.destination.clone();
        destination.push(self.game.strip_prefix(&self.root)?);
        remove_full_extension(&mut destination);

        let directory = destination.parent().unwrap();

        let prefix = self.game.file_name().unwrap();

        static RE: OnceLock<Regex> = OnceLock::new();
        let re = RE
            .get_or_init(|| Regex::new(r"(?:srm|state[0-9]*|state\.auto|sav|rtc|ldci)$").unwrap());

        for file in files_for_directory(directory) {
            let basename = PathBuf::from(file.file_name().unwrap());
            let mut stem = basename.clone();
            remove_full_extension(&mut stem);
            if stem != prefix {
                continue;
            }

            let Some(extension) = full_extension(&file) else {
                continue;
            };
            if re.is_match(extension) {
                info!("Deleting file {file:?} for having extension {extension:?}");
                std::fs::remove_file(&file)?;
                results.push_back(format!("Removed {basename:?}"));
            }
        }

        let extension = match full_extension(current_save) {
            Some("state") => "state.auto",
            Some(e) => e,
            None => return Err(anyhow!("Couldn't extract extension")),
        };
        destination.set_extension(extension);
        info!("Copying {current_save:?} into {destination:?}");
        std::fs::copy(current_save, &destination)?;

        results.push_front(format!("Copied {label}"));

        Ok(Vec::from(results))
    }
}

impl Scene<Operation> for SelectSave {
    fn pressed(&mut self, button: &Button) -> Option<Action<Operation>> {
        if let Some(action) = self.list.handle_navigation(button) {
            return Some(action);
        }

        match button {
            Button::A => {
                let scene = match self.commit_save() {
                    Ok(messages) => Message::new(messages, false),
                    Err(e) => {
                        Message::new(vec![format!("Error updating saves"), e.to_string()], true)
                    }
                };
                Some(Action::Push(Box::new(scene)))
            }
            Button::Start => Some(Action::Complete(Operation::ExecGame(self.game.clone()))),
            _ => Some(Action::Continue),
        }
    }

    fn draw(&self, screen: &mut Screen) {
        let (current_save, preview) = self.current_save();
        let (_, body_height) = screen.measure_text(FontSize::Body, "0");
        let gap = screen.recommended_margin();
        let (screen_width, _) = screen.size();
        let preview_width = preview_width_for_screen_width(screen_width);

        let draw_metadata = |screen: &mut Screen, preview_height: u32| {
            let metadata = self.metadata_for_save(current_save);

            let h = if preview_height == 0 {
                0
            } else {
                preview_height + gap
            };

            screen.draw_rect(
                Color::RGBA(0, 0, 0, 255),
                Rect::new(
                    (screen_width - preview_width - gap * 3) as i32,
                    gap as i32,
                    preview_width + gap * 2,
                    h + gap * 2 + metadata.len() as u32 * (body_height + PADDING * 2 - 1),
                ),
            );

            let mut y = (preview_height + gap * 3) as i32;
            for metadatum in metadata {
                let (w, h) = screen.measure_text(FontSize::Body, &metadatum);
                let w = min(w, preview_width + gap * 2);
                let x = (screen_width - preview_width - gap * 2) as i32
                    + (preview_width as i32 - w as i32) / 2;
                screen.draw_text_clipped(FontSize::Body, &metadatum, x, y, w);
                y += (h + PADDING * 2 - 1) as i32;
            }
        };

        if let Some(path) = preview {
            let (width, height) = self.draw_stylized_background(
                screen,
                path,
                self.list.cursor().index() + self.offset,
            );
            let preview_height = (preview_width as f32 * (height as f32) / (width as f32)) as u32;

            draw_metadata(screen, preview_height);

            screen.draw_image(
                path,
                None,
                Some(Rect::new(
                    (screen_width - preview_width - gap * 2) as i32,
                    gap as i32 * 2,
                    preview_width,
                    preview_height,
                )),
            );
        } else {
            draw_metadata(screen, 0);
        }

        self.list
            .draw(screen, true, true, |(save, _)| self.label_for(save));
    }
}
