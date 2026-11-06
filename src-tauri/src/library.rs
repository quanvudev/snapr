use crate::State;
use crate::capture;
use crate::config::Config;
use crate::gallery::{self, Kind};
use crate::transcript::{self, Track};
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

const IMAGES: &str = "Images";
const VIDEOS: &str = "Videos";

pub fn images(cfg: &Config) -> PathBuf {
    cfg.dir().join(IMAGES)
}

pub fn videos(cfg: &Config) -> PathBuf {
    cfg.dir().join(VIDEOS)
}

pub fn folders(cfg: &Config) -> [PathBuf; 3] {
    [cfg.dir(), images(cfg), videos(cfg)]
}

pub fn migrate(app: &AppHandle) {
    let state = app.state::<State>();
    let root = {
        let cfg = state.cfg.read().unwrap();
        if cfg.folders_migrated {
            return;
        }
        cfg.dir()
    };
    if let Some(root) = root.canonicalize().ok().and_then(|root| gallery::plain(&root)) {
        for (old, new) in move_into(&root) {
            transcript::languages::rename(app, &old, &new);
        }
    }
    let mut cfg = state.cfg.write().unwrap();
    cfg.folders_migrated = true;
    let _ = cfg.save(&state.path);
}

fn move_into(root: &Path) -> Vec<(PathBuf, PathBuf)> {
    let Ok(entries) = std::fs::read_dir(root) else { return Vec::new() };
    let files: Vec<PathBuf> = entries
        .flatten()
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_file()))
        .map(|entry| entry.path())
        .collect();
    let mut moved = Vec::new();
    for file in files {
        let folder = match Kind::of(&file) {
            Kind::Photo => IMAGES,
            Kind::Video => VIDEOS,
            Kind::Other => continue,
        };
        let (Some(stem), Some(ext)) = (file.file_stem().and_then(|s| s.to_str()), file.extension().and_then(|s| s.to_str()))
        else {
            continue;
        };
        let dir = root.join(folder);
        if std::fs::create_dir_all(&dir).is_err() {
            continue;
        }
        let target = capture::stem_path(&dir, stem, ext);
        if std::fs::rename(&file, &target).is_err() {
            continue;
        }
        if folder == VIDEOS {
            for track in Track::all() {
                let from = track.path(&file);
                let to = track.path(&target);
                if from.is_file() && !to.exists() {
                    let _ = std::fs::rename(&from, &to);
                }
            }
            moved.push((file, target));
        }
    }
    moved
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn moves_into_folders() {
        let root = std::env::temp_dir().join(format!("snapr-library-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join(VIDEOS)).unwrap();
        for name in ["a.png", "b.gif", "c.mp4", "c.srt", "c.en.srt", "notes.txt", "Videos/c.mp4"] {
            fs::write(root.join(name), name).unwrap();
        }
        let moved = move_into(&root);
        assert_eq!(moved, vec![(root.join("c.mp4"), root.join(VIDEOS).join("c_1.mp4"))]);
        for name in ["Images/a.png", "Images/b.gif", "Videos/c.mp4", "Videos/c_1.mp4", "Videos/c_1.srt", "Videos/c_1.en.srt", "notes.txt"] {
            assert!(root.join(name).is_file(), "{name}");
        }
        for name in ["a.png", "b.gif", "c.mp4", "c.srt", "c.en.srt"] {
            assert!(!root.join(name).exists(), "{name}");
        }
        assert_eq!(fs::read_to_string(root.join("Videos/c_1.srt")).unwrap(), "c.srt");
        assert!(move_into(&root).is_empty());
        let _ = fs::remove_dir_all(&root);
    }
}
