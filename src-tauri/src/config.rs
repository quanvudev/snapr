use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tauri_plugin_global_shortcut::Shortcut;

#[derive(Clone, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct Hotkeys {
    pub modifiers: String,
    pub fullscreen: String,
    pub window: String,
    pub area: String,
    #[serde(default)]
    pub text: String,
    pub record: String,
    #[serde(default)]
    pub draw: String,
    #[serde(default)]
    pub ring: String,
}

pub const SLOTS: usize = 7;
pub const TEXT_SLOT: usize = 3;
pub const DRAW_SLOT: usize = 5;
pub const RING_SLOT: usize = 6;

pub fn skipped(i: usize, combos: &[String; SLOTS], text_active: bool) -> bool {
    match i {
        TEXT_SLOT => !text_active || combos[i].is_empty(),
        DRAW_SLOT | RING_SLOT => combos[i].is_empty(),
        _ => false,
    }
}

#[derive(Clone, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct Padding {
    pub enabled: bool,
    pub size: u32,
    pub image: String,
    pub round: bool,
    pub radius: u32,
}

impl Default for Padding {
    fn default() -> Self {
        Self { enabled: false, size: 32, image: String::new(), round: false, radius: 12 }
    }
}

#[derive(Clone, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    pub output_dir: String,
    pub copy_to_clipboard: bool,
    pub preview_seconds: u32,
    pub language: String,
    pub theme: String,
    pub record_fps: u32,
    pub text_capture: bool,
    pub padding: Padding,
    pub hotkeys: Hotkeys,
}

impl Default for Hotkeys {
    fn default() -> Self {
        Self {
            modifiers: "Win+Shift".into(),
            fullscreen: "1".into(),
            window: "2".into(),
            area: "3".into(),
            text: "4".into(),
            record: "5".into(),
            draw: "6".into(),
            ring: String::new(),
        }
    }
}

impl Hotkeys {
    pub fn combos(&self) -> [String; SLOTS] {
        [&self.fullscreen, &self.window, &self.area, &self.text, &self.record, &self.draw, &self.ring].map(|key| {
            if key.is_empty() || key.contains('+') || is_mouse(key) || self.modifiers.is_empty() {
                key.clone()
            } else {
                format!("{}+{key}", self.modifiers)
            }
        })
    }
}

pub fn is_mouse(text: &str) -> bool {
    crate::mouse::Binding::parse(text).is_some()
}

pub fn parse_hotkey(text: &str) -> Result<Shortcut, String> {
    let mapped: Vec<&str> = text
        .split('+')
        .map(|t| if t.eq_ignore_ascii_case("win") { "Super" } else { t })
        .collect();
    mapped.join("+").parse::<Shortcut>().map_err(|e| e.to_string())
}

impl Default for Config {
    fn default() -> Self {
        Self {
            output_dir: "~/Pictures/snapr".into(),
            copy_to_clipboard: true,
            preview_seconds: 2,
            language: "vi".into(),
            theme: "system".into(),
            record_fps: 30,
            text_capture: true,
            padding: Padding::default(),
            hotkeys: Hotkeys::default(),
        }
    }
}

impl Config {
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let text = toml::to_string_pretty(self).map_err(std::io::Error::other)?;
        std::fs::write(path, text)
    }

    pub fn load(path: &Path) -> Self {
        let cfg: Self = std::fs::read_to_string(path)
            .ok()
            .and_then(|s| toml::from_str(&s).ok())
            .unwrap_or_default();
        let _ = std::fs::create_dir_all(cfg.dir());
        cfg
    }

    pub fn fps(&self) -> u32 {
        if self.record_fps == 60 { 60 } else { 30 }
    }

    pub fn dir(&self) -> PathBuf {
        let raw = self.output_dir.replace('/', "\\");
        match raw.strip_prefix('~') {
            Some(rest) => dirs::home_dir().unwrap_or_default().join(rest.trim_start_matches('\\')),
            None => PathBuf::from(raw),
        }
    }
}
