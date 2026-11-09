use crate::ring::{AppEntry, Wedge, key_char};
use crate::video::Quality;
use serde::de::IgnoredAny;
use serde::{Deserialize, Deserializer, Serialize};
use std::path::{Path, PathBuf};
use tauri_plugin_global_shortcut::Shortcut;

pub const MAX_RING_ITEMS: usize = 12;
const COMPRESS_HEIGHT_FULL_HD: u32 = 1080;
const COMPRESS_HEIGHT_HD: u32 = 720;
const TRANSCRIPT_AUTO: &str = "auto";
const TRANSCRIPT_LANGUAGES: [&str; 3] = [TRANSCRIPT_AUTO, "vi", "en"];

#[derive(Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RingItem {
    Builtin {
        id: Wedge,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        key: Option<String>,
    },
    App(AppEntry),
}

impl RingItem {
    fn normalise_key(&mut self) {
        let key = match self {
            RingItem::Builtin { key, .. } => key,
            RingItem::App(app) => &mut app.key,
        };
        *key = key.as_deref().and_then(key_char).map(String::from);
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum LenientItem {
    Known(RingItem),
    Unknown(IgnoredAny),
}

fn lenient_items<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<Vec<RingItem>>, D::Error> {
    let raw = Option::<Vec<LenientItem>>::deserialize(deserializer)?;
    Ok(raw.map(|items| {
        items
            .into_iter()
            .filter_map(|item| match item {
                LenientItem::Known(mut item) => {
                    item.normalise_key();
                    Some(item)
                }
                LenientItem::Unknown(_) => None,
            })
            .collect()
    }))
}

#[derive(Clone, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct Ring {
    pub enabled: bool,
    pub keys: bool,
    pub idle_close: u32,
    #[serde(deserialize_with = "lenient_items", skip_serializing_if = "Option::is_none")]
    pub items: Option<Vec<RingItem>>,
}

impl Default for Ring {
    fn default() -> Self {
        Self { enabled: true, keys: true, idle_close: 5, items: None }
    }
}

impl Ring {
    pub fn sanitise(&mut self) {
        let Some(items) = self.items.as_mut() else { return };
        items.iter_mut().for_each(RingItem::normalise_key);
        let mut seen = Vec::new();
        items.retain(|item| match item {
            RingItem::App(app) => !app.path.trim().is_empty(),
            RingItem::Builtin { id, .. } if seen.contains(id) => false,
            RingItem::Builtin { id, .. } => {
                seen.push(*id);
                true
            }
        });
        items.truncate(MAX_RING_ITEMS);
    }
}

#[derive(Clone, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum HotkeyMode {
    #[default]
    Fixed,
    Manual,
}

#[derive(Clone, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct Hotkeys {
    pub mode: HotkeyMode,
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
    pub manual: [String; SLOTS],
}

pub const SLOTS: usize = 7;
pub const TEXT_SLOT: usize = 3;
pub const DRAW_SLOT: usize = 5;
pub const RING_SLOT: usize = 6;

pub fn skipped(i: usize, combos: &[String; SLOTS], text_active: bool, ring_enabled: bool) -> bool {
    match i {
        TEXT_SLOT => !text_active || combos[i].is_empty(),
        RING_SLOT => !ring_enabled || combos[i].is_empty(),
        DRAW_SLOT => combos[i].is_empty(),
        _ => false,
    }
}

#[derive(Clone, Copy, Default, PartialEq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RecordMode {
    #[default]
    Fullscreen,
    Window,
    Area,
    Captions,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct CameraLook {
    pub strength: u32,
    pub smoothing: u32,
    pub edge: u32,
    pub feather: u32,
    pub wrap: u32,
    pub wrap_radius: u32,
}

impl Default for CameraLook {
    fn default() -> Self {
        Config::default().camera_look()
    }
}

fn default_blur_strength() -> u32 {
    4
}

fn default_blur_smoothing() -> u32 {
    50
}

fn default_bg_edge() -> u32 {
    30
}

fn default_bg_feather() -> u32 {
    1
}

fn default_bg_wrap() -> u32 {
    50
}

fn default_bg_wrap_radius() -> u32 {
    4
}

#[derive(Clone, Copy, Default, PartialEq, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CameraEffect {
    #[default]
    None,
    Blur,
    Image,
}

#[derive(Clone, Copy, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OcrEngine {
    #[default]
    Windows,
    Paddle,
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PaddleSize {
    Tiny,
    #[default]
    Small,
    Medium,
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum WhisperModel {
    Base,
    #[default]
    Small,
    Turbo,
}

#[derive(Clone, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct Padding {
    pub enabled: bool,
    pub size: u32,
    pub image: String,
    pub round: bool,
    pub radius: u32,
    pub wallpaper: bool,
}

impl Default for Padding {
    fn default() -> Self {
        Self { enabled: false, size: 32, image: String::new(), round: false, radius: 12, wallpaper: false }
    }
}

#[derive(Clone, Copy, PartialEq, Default)]
pub enum RedactStyle {
    #[default]
    Black,
    Pixelate,
}

#[derive(Clone, PartialEq, Deserialize, Serialize)]
#[serde(default)]
pub struct Redact {
    pub auto_capture: bool,
    pub engine: String,
    pub style: String,
    pub email: bool,
    pub phone: bool,
    pub card: bool,
    pub ipv4: bool,
    pub secrets: bool,
    pub patterns: Vec<String>,
}

impl Default for Redact {
    fn default() -> Self {
        Self {
            auto_capture: false,
            engine: "windows".into(),
            style: "black".into(),
            email: true,
            phone: true,
            card: true,
            ipv4: true,
            secrets: true,
            patterns: Vec::new(),
        }
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
    #[serde(default)]
    pub record_countdown: bool,
    #[serde(default = "default_mic")]
    pub record_mic: String,
    #[serde(default)]
    pub record_camera: String,
    #[serde(default)]
    pub record_speaker: String,
    pub record_denoise: bool,
    #[serde(default)]
    pub record_camera_effect: CameraEffect,
    #[serde(default)]
    pub record_camera_image: String,
    #[serde(default = "default_blur_strength")]
    pub record_blur_strength: u32,
    #[serde(default = "default_blur_smoothing")]
    pub record_blur_smoothing: u32,
    #[serde(default = "default_bg_edge")]
    pub record_bg_edge: u32,
    #[serde(default = "default_bg_feather")]
    pub record_bg_feather: u32,
    #[serde(default = "default_bg_wrap")]
    pub record_bg_wrap: u32,
    #[serde(default = "default_bg_wrap_radius")]
    pub record_bg_wrap_radius: u32,
    #[serde(default)]
    pub record_mode: RecordMode,
    #[serde(default = "default_quality")]
    pub record_quality: String,
    #[serde(default)]
    pub record_compress: bool,
    #[serde(default = "default_compress_height")]
    pub record_compress_height: u32,
    pub text_capture: bool,
    #[serde(default)]
    pub ocr_engine: OcrEngine,
    #[serde(default)]
    pub paddle_model: PaddleSize,
    #[serde(default)]
    pub record_transcribe: bool,
    #[serde(default)]
    pub record_translate: bool,
    #[serde(default)]
    pub hymt_license_accepted: bool,
    #[serde(default)]
    pub whisper_model: WhisperModel,
    #[serde(default = "default_transcript_language")]
    pub transcript_language: String,
    #[serde(default = "default_acceleration")]
    pub ai_acceleration: String,
    pub update_check: bool,
    pub padding: Padding,
    pub redact: Redact,
    pub hotkeys: Hotkeys,
    pub ring: Ring,
    #[serde(default)]
    pub folders_migrated: bool,
    #[serde(default)]
    pub captions_language: String,
    #[serde(default)]
    pub captions_target: String,
}

pub const CAPTIONS_OFF: &str = "off";

fn default_mic() -> String {
    "default".into()
}

fn default_transcript_language() -> String {
    TRANSCRIPT_AUTO.into()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Acceleration {
    Auto,
    Cpu,
    DirectMl,
    Cuda,
}

impl Acceleration {
    pub fn parse(text: &str) -> Self {
        match text {
            "cpu" => Self::Cpu,
            "directml" => Self::DirectMl,
            "cuda" => Self::Cuda,
            _ => Self::Auto,
        }
    }
}

fn default_acceleration() -> String {
    "auto".into()
}

fn default_quality() -> String {
    "medium".into()
}

fn default_compress_height() -> u32 {
    COMPRESS_HEIGHT_FULL_HD
}

impl Default for Hotkeys {
    fn default() -> Self {
        Self {
            mode: HotkeyMode::Fixed,
            modifiers: "Ctrl+Shift".into(),
            fullscreen: "1".into(),
            window: "2".into(),
            area: "3".into(),
            text: "4".into(),
            record: "5".into(),
            draw: "6".into(),
            ring: String::new(),
            manual: Default::default(),
        }
    }
}

impl Hotkeys {
    pub fn combos(&self) -> [String; SLOTS] {
        match self.mode {
            HotkeyMode::Fixed => {
                [&self.fullscreen, &self.window, &self.area, &self.text, &self.record, &self.draw, &self.ring].map(|key| {
                    if key.is_empty() || key.contains('+') || is_mouse(key) || self.modifiers.is_empty() {
                        key.clone()
                    } else {
                        format!("{}+{key}", self.modifiers)
                    }
                })
            }
            HotkeyMode::Manual => self.manual.clone(),
        }
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
            record_countdown: false,
            record_mic: default_mic(),
            record_camera: String::new(),
            record_speaker: String::new(),
            record_denoise: true,
            record_camera_effect: CameraEffect::None,
            record_camera_image: String::new(),
            record_blur_strength: default_blur_strength(),
            record_blur_smoothing: default_blur_smoothing(),
            record_bg_edge: default_bg_edge(),
            record_bg_feather: default_bg_feather(),
            record_bg_wrap: default_bg_wrap(),
            record_bg_wrap_radius: default_bg_wrap_radius(),
            record_mode: RecordMode::Fullscreen,
            record_quality: default_quality(),
            record_compress: false,
            record_compress_height: default_compress_height(),
            text_capture: true,
            ocr_engine: OcrEngine::Windows,
            paddle_model: PaddleSize::Small,
            record_transcribe: false,
            record_translate: false,
            hymt_license_accepted: false,
            whisper_model: WhisperModel::Small,
            transcript_language: default_transcript_language(),
            ai_acceleration: default_acceleration(),
            update_check: true,
            padding: Padding::default(),
            redact: Redact::default(),
            hotkeys: Hotkeys::default(),
            ring: Ring::default(),
            folders_migrated: true,
            captions_language: String::new(),
            captions_target: String::new(),
        }
    }
}

impl Config {
    pub fn ring_items(&self) -> Vec<RingItem> {
        let mut items = self
            .ring
            .items
            .clone()
            .unwrap_or_else(|| Wedge::ALL.into_iter().map(|id| RingItem::Builtin { id, key: None }).collect());
        items.truncate(MAX_RING_ITEMS);
        items
    }

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

    pub fn camera_effect(&self) -> CameraEffect {
        match self.record_camera_effect {
            CameraEffect::Image if self.record_camera_image.is_empty() => CameraEffect::None,
            effect => effect,
        }
    }

    pub fn camera_look(&self) -> CameraLook {
        CameraLook {
            strength: self.record_blur_strength.clamp(1, 10),
            smoothing: self.record_blur_smoothing.min(90),
            edge: self.record_bg_edge.clamp(10, 60),
            feather: self.record_bg_feather.min(4),
            wrap: self.record_bg_wrap.min(100),
            wrap_radius: self.record_bg_wrap_radius.clamp(1, 10),
        }
    }

    pub fn fps(&self) -> u32 {
        if self.record_fps == 60 { 60 } else { 30 }
    }

    pub fn compress_height(&self) -> u32 {
        if self.record_compress_height == COMPRESS_HEIGHT_HD { COMPRESS_HEIGHT_HD } else { COMPRESS_HEIGHT_FULL_HD }
    }

    pub fn transcript_language(&self) -> &str {
        TRANSCRIPT_LANGUAGES.into_iter().find(|&l| l == self.transcript_language).unwrap_or(TRANSCRIPT_AUTO)
    }

    pub fn captions_language(&self) -> crate::captions::Language {
        crate::captions::Language::parse(&self.captions_language).unwrap_or(crate::captions::Language::En)
    }

    pub fn captions_target(&self) -> Option<&'static str> {
        let source = self.captions_language().code();
        match self.captions_target.as_str() {
            CAPTIONS_OFF => None,
            code => crate::translate::languages::find(code)
                .map(|l| l.code)
                .or_else(|| Some(crate::translate::languages::default_target(source, &self.language))),
        }
    }

    pub fn redact_paddle(&self) -> bool {
        self.redact.engine == "paddle"
    }

    pub fn redact_style(&self) -> RedactStyle {
        match self.redact.style.as_str() {
            "pixelate" => RedactStyle::Pixelate,
            _ => RedactStyle::Black,
        }
    }

    pub fn acceleration(&self) -> Acceleration {
        Acceleration::parse(&self.ai_acceleration)
    }

    pub fn quality(&self) -> Quality {
        Quality::parse(&self.record_quality).unwrap_or_default()
    }

    pub fn dir(&self) -> PathBuf {
        let raw = self.output_dir.replace('/', "\\");
        match raw.strip_prefix('~') {
            Some(rest) => dirs::home_dir().unwrap_or_default().join(rest.trim_start_matches('\\')),
            None => PathBuf::from(raw),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acceleration_defaults_to_auto() {
        let old: Config = toml::from_str("output_dir = \"~/Pictures/snapr\"").unwrap();
        assert_eq!(old.acceleration(), Acceleration::Auto);
        let unknown = Config { ai_acceleration: "tpu".into(), ..Config::default() };
        assert_eq!(unknown.acceleration(), Acceleration::Auto);
        let cuda = Config { ai_acceleration: "cuda".into(), ..Config::default() };
        assert_eq!(cuda.acceleration(), Acceleration::Cuda);
    }

    #[test]
    fn camera_effect_defaults_to_none() {
        let old: Config = toml::from_str("").unwrap();
        assert_eq!(old.record_camera_effect, CameraEffect::None);
        let blur: Config = toml::from_str("record_camera_effect = \"blur\"").unwrap();
        assert_eq!(blur.record_camera_effect, CameraEffect::Blur);
        assert_eq!(
            old.camera_look(),
            CameraLook { strength: 4, smoothing: 50, edge: 30, feather: 1, wrap: 50, wrap_radius: 4 }
        );
        let wild: Config = toml::from_str(
            "record_blur_strength = 99\nrecord_blur_smoothing = 100\nrecord_bg_edge = 99\nrecord_bg_feather = 9\nrecord_bg_wrap = 200\nrecord_bg_wrap_radius = 50",
        )
        .unwrap();
        assert_eq!(
            wild.camera_look(),
            CameraLook { strength: 10, smoothing: 90, edge: 60, feather: 4, wrap: 100, wrap_radius: 10 }
        );
        let low: Config = toml::from_str("record_blur_strength = 0\nrecord_bg_edge = 0\nrecord_bg_wrap_radius = 0").unwrap();
        let look = low.camera_look();
        assert_eq!((look.strength, look.edge, look.wrap_radius), (1, 10, 1));
        let no_image: Config = toml::from_str("record_camera_effect = \"image\"").unwrap();
        assert_eq!(no_image.camera_effect(), CameraEffect::None);
        let image: Config =
            toml::from_str("record_camera_effect = \"image\"
record_camera_image = \"C:/bg.png\"").unwrap();
        assert_eq!(image.camera_effect(), CameraEffect::Image);
    }

    #[test]
    fn captions_default_to_english() {
        let old: Config = toml::from_str("record_mode = \"area\"").unwrap();
        assert_eq!(old.captions_language(), crate::captions::Language::En);
        let vi_ui = Config { language: "vi".into(), ..old.clone() };
        assert_eq!(vi_ui.captions_target(), Some("vi"));
        let japanese = Config { captions_language: "ja".into(), captions_target: "vi".into(), ..old.clone() };
        assert_eq!((japanese.captions_language(), japanese.captions_target()), (crate::captions::Language::Ja, Some("vi")));
        let off = Config { captions_target: CAPTIONS_OFF.into(), ..old.clone() };
        assert_eq!(off.captions_target(), None);
        let captions: Config = toml::from_str("record_mode = \"captions\"").unwrap();
        assert_eq!(captions.record_mode, RecordMode::Captions);
    }

    #[test]
    fn denoise_defaults_to_on() {
        let old: Config = toml::from_str("output_dir = \"~/Pictures/snapr\"
record_mic = \"default\"").unwrap();
        assert!(old.record_denoise);
        let off: Config = toml::from_str("record_denoise = false").unwrap();
        assert!(!off.record_denoise);
    }
}
