use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn full_extension(path: &Path) -> Option<&str> {
    path.file_name().and_then(|b| {
        b.to_str()
            .and_then(|b| b.split_once('.').map(|(_, after)| after))
    })
}

pub fn remove_full_extension(path: &mut PathBuf) {
    if let Some(basename) = path.file_name() {
        let basename = basename.to_owned();
        if let Some(extension) = full_extension(path) {
            if let Some(basename) = basename.to_str() {
                let stem_len = basename.len() - extension.len() - 1;
                if stem_len > 0 {
                    let stem = &basename[0..stem_len];
                    path.set_file_name(stem);
                }
            }
        }
    }
}

pub fn files_for_directory(directory: &Path) -> impl Iterator<Item = PathBuf> {
    WalkDir::new(directory)
        .min_depth(1)
        .max_depth(1)
        .sort_by_file_name()
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .map(|e| e.into_path())
}
