use super::{
    message::Message,
    saves_for_game,
    selectgame::{PADDING, PAGE_SIZE},
    Action, Scene,
};
use crate::{
    cursor::Cursor,
    extractor::Extractor,
    internal::{full_extension, remove_full_extension},
    ui::{
        screen::{FontSize, Screen, SHADOW_DELTA},
        Button,
    },
};
use anyhow::{anyhow, Result};
use chrono::prelude::*;
use itertools::Itertools;
use lazy_static::lazy_static;
use log::{debug, info};
use rand::Rng;
use regex::Regex;
use sdl2::{pixels::Color, rect::Rect};
use std::path::{Path, PathBuf};
use std::{cmp::min, collections::VecDeque};

pub(super) struct SelectSave {
    game: PathBuf,
    root: PathBuf,
    destination: PathBuf,
    saves: Vec<(PathBuf, Option<PathBuf>)>,
    cursor: Cursor,
    offset: usize,
    extractor: Option<Extractor>,
}

impl SelectSave {
    pub(super) fn new(game: PathBuf, root: PathBuf, destination: PathBuf) -> Self {
        let offset = rand::thread_rng().gen_range(100..999);
        let saves = saves_for_game(&game);
        let len = saves.len();

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
            saves,
            root,
            destination,
            offset,
            cursor: Cursor::new(len, PAGE_SIZE),
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

        lazy_static! {
            static ref RE: Regex = Regex::new(
                r"^([0-9][0-9][0-9][0-9])([0-9][0-9])([0-9][0-9])-([0-9][0-9])([0-9][0-9])([0-9][0-9])$",
            ).unwrap();
        }
        let caps = RE.captures(stem)?;

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
        } else {
            debug!("Could not produce duration for save: {save:?}");
        }

        if let Ok(meta) = self.extract_save(save) {
            metadata.extend(meta);
        }

        metadata
    }

    fn current_save(&self) -> &(PathBuf, Option<PathBuf>) {
        &self.saves[self.cursor.index()]
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

        lazy_static! {
            static ref RE: Regex =
                Regex::new(r"(?:srm|state[0-9]*|state\.auto|sav|rtc|ldci)$").unwrap();
        }

        for file in walkdir::WalkDir::new(directory)
            .min_depth(1)
            .max_depth(1)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
            .map(|e| e.into_path())
        {
            let basename = PathBuf::from(file.file_name().unwrap());
            let mut stem = basename.clone();
            remove_full_extension(&mut stem);
            if stem != prefix {
                continue;
            }

            let Some(extension) = full_extension(&file) else { continue };
            if RE.is_match(extension) {
                info!("Deleting file {file:?} for having extension {extension:?}");
                std::fs::remove_file(&file)?;
                results.push_back(format!("Removed {basename:?}"));
            } else {
                debug!("Ignoring file {file:?} for having extension {extension:?}");
            }
        }

        let extension = match full_extension(current_save) {
            Some(e) if e == "state" => "state.auto",
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

pub fn preview_width_for_screen_width(width: u32) -> u32 {
    width / 3
}

impl Scene for SelectSave {
    fn pressed(&mut self, button: &Button) -> Action {
        match button {
            Button::B => return Action::Pop,
            Button::Up => {
                self.cursor.up();
            }
            Button::Down => {
                self.cursor.down();
            }
            Button::Left => {
                self.cursor.page_up();
            }
            Button::Right => {
                self.cursor.page_down();
            }
            Button::A => {
                let scene = match self.commit_save() {
                    Ok(messages) => Message::new(messages, false),
                    Err(e) => {
                        Message::new(vec![format!("Error updating saves"), e.to_string()], true)
                    }
                };
                return Action::Push(Box::new(scene));
            }
            Button::Start => {
                return Action::ExecGame(&self.game);
            }
            _ => {}
        }

        Action::Continue
    }

    fn draw(&self, screen: &mut Screen) {
        let mut y = 0;
        let (current_save, preview) = self.current_save();

        let (_, font18height) = screen.measure_text(FontSize::Size18, "S");
        let (_, font14height) = screen.measure_text(FontSize::Size14, "0");

        let gap = screen.recommended_margin();
        let (screen_width, screen_height) = screen.size();
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
                    h + gap * 2 + metadata.len() as u32 * (font14height + PADDING * 2 - 1),
                ),
            );

            let mut y = (preview_height + gap * 3) as i32;
            for metadatum in metadata {
                let (w, h) = screen.measure_text(FontSize::Size14, &metadatum);
                let w = min(w, preview_width + gap * 2);
                let x = (screen_width - preview_width - gap * 2) as i32
                    + (preview_width as i32 - w as i32) / 2;
                debug!("Drawing metadata \"{metadatum:?}\" ({w:?}x{h:?}) at ({x:?}, {y:?})");
                screen.draw_text_clipped(FontSize::Size14, &metadatum, x, y, w);
                y += (h + PADDING * 2 - 1) as i32;
            }
        };

        if let Some(path) = preview {
            let background = Screen::create_background(path, 128, 90);
            let (width, height) = background.size();
            let i = (self.cursor.index() + self.offset) as f64;
            let scale = (2.0 + i.sin() / 2.0) as f32;
            let angle = i.cos() * 30.0;
            screen.draw_background(background, scale, angle);

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

        let full_list_height =
            2 * gap + font18height + PAGE_SIZE as u32 * (font14height + PADDING * 2 - 1);
        let fill_height = screen_height - 2 * gap;
        let extra_gap = fill_height - full_list_height;

        let bg_height = 2 * gap
            + font18height
            + extra_gap
            + self.cursor.visible_items() as u32 * (font14height + PADDING * 2 - 1);

        screen.draw_rect(
            Color::RGBA(0, 0, 0, 64),
            Rect::new(
                gap as i32,
                gap as i32,
                screen_width - 5 * gap - preview_width,
                bg_height,
            ),
        );

        screen.draw_text(
            FontSize::Size18,
            "Select a save",
            2 * gap as i32,
            2 * gap as i32,
        );
        y += (2 * gap + font18height + extra_gap) as i32;

        for (selected, (save, _)) in self.cursor.iter(self.saves.iter()) {
            if selected {
                screen.draw_rect(
                    Color::RGBA(0, 0, 255, 180),
                    Rect::new(
                        2 * gap as i32,
                        y - PADDING as i32,
                        screen_width - 7 * gap - preview_width,
                        14 + 2 * PADDING + SHADOW_DELTA,
                    ),
                );
            }

            let label = self.label_for(save);
            screen.draw_text_clipped(
                FontSize::Size14,
                &label,
                2 * gap as i32 + PADDING as i32,
                y,
                screen_width - 7 * gap - preview_width - 2 * PADDING,
            );
            y += (font14height + PADDING * 2 - 1) as i32;
        }
    }
}
