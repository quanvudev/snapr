pub mod segment;
pub mod session;
pub mod stream;
pub mod tokens;

use crate::State;
use crate::events;
use crate::fetch::{self, Fetch, Remote};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};

const STORE_DIR: &str = "captions";
const THROTTLE: Duration = Duration::from_millis(100);
const LATIN_TOKENS_PER_SECOND: f32 = 6.5;
const DENSE_TOKENS_PER_SECOND: f32 = 13.0;

macro_rules! base {
    () => {
        "https://huggingface.co/moonshine-ai/moonshine-voice-assets/resolve/015f7269048251aff164cb60df02bfea76884c8f/model/"
    };
}

const EN: [Remote; 8] = [
    Remote { url: concat!(base!(), "small-streaming-en/quantized_26_11_07/adapter.ort"), sha256: "100d4806a3ba19b739fa9ae5c952e4d5267983090eea4a085e08d3e0bfd6875b", bytes: 2_870_320 },
    Remote { url: concat!(base!(), "small-streaming-en/quantized_26_11_07/cross_kv.ort"), sha256: "aa0ed033b7065ef744c36dce00228c708b43f2f2410e8ec782a960105c8595f7", bytes: 5_356_496 },
    Remote { url: concat!(base!(), "small-streaming-en/quantized_26_11_07/decoder_kv.ort"), sha256: "e4fae3869851b861177b5e1619c77ec9262365b2747ec322298d11753e88b38d", bytes: 82_043_616 },
    Remote { url: concat!(base!(), "small-streaming-en/quantized_26_11_07/encoder.ort"), sha256: "3d8f4b778135111f5c7653a7cc258e5809ef9fa9124bc85c03598c251460d012", bytes: 44_144_096 },
    Remote { url: concat!(base!(), "small-streaming-en/quantized_26_11_07/frontend.model.ort"), sha256: "4a8d5ef7e99a5e3011dbf09ffac4176056587f2aec20f885d606b9f55fbed408", bytes: 26_992 },
    Remote { url: concat!(base!(), "small-streaming-en/quantized_26_11_07/frontend.weights.ort"), sha256: "5379f3dfebe849c6cbacf84f0aa7441956f9572efe1720d44aa267e7dff93056", bytes: 7_769_416 },
    Remote { url: concat!(base!(), "small-streaming-en/quantized_26_11_07/streaming_config.json"), sha256: "26f02b6afb22d60871a5efd85c3d38e569cc0ddb6c5eb6e93d3260152ae8a47a", bytes: 512 },
    Remote { url: concat!(base!(), "small-streaming-en/quantized_26_11_07/tokenizer.bin"), sha256: "6884b35fd6377d4c4d32336a0bc152f36b64d1e45b6503683cdc238250a8472d", bytes: 249_974 },
];
const VI: [Remote; 8] = [
    Remote { url: concat!(base!(), "tiny-streaming-vi/quantized_26_11_07/adapter.ort"), sha256: "8351344095ce2e7e8dca7a932c102c7976fd5031d9f50946c05fe4e8ea520cab", bytes: 1_318_472 },
    Remote { url: concat!(base!(), "tiny-streaming-vi/quantized_26_11_07/cross_kv.ort"), sha256: "0a245cc6b617f9c3a2aebe3ba5e4e12dc6118750fb7e478ccb95fe629abe56cf", bytes: 1_288_120 },
    Remote { url: concat!(base!(), "tiny-streaming-vi/quantized_26_11_07/decoder_kv.ort"), sha256: "b398871f7668c667f9853758e1bd5ffb77d1ddf72f90ae73ecd4b1f05b3df1d9", bytes: 19_717_336 },
    Remote { url: concat!(base!(), "tiny-streaming-vi/quantized_26_11_07/encoder.ort"), sha256: "832ee6f223d0e0e7a5ae6d2ee4edeaabb72740dee44530bf2d1e538d4ff935ea", bytes: 7_674_856 },
    Remote { url: concat!(base!(), "tiny-streaming-vi/quantized_26_11_07/frontend.model.ort"), sha256: "fb8c8ae3cbc72f28e56a6f7577440b4e41e41a86b183d50ebd3d35215b712a7d", bytes: 23_176 },
    Remote { url: concat!(base!(), "tiny-streaming-vi/quantized_26_11_07/frontend.weights.ort"), sha256: "beb51a94a57cb33ea740c23d7841ca2855e39fd9df7468a407dfd5eea471d8ca", bytes: 2_093_280 },
    Remote { url: concat!(base!(), "tiny-streaming-vi/quantized_26_11_07/streaming_config.json"), sha256: "d31afd7c2dd5ee9ae3fa232312c5b42cb3e4fe1c6e9c53d885f2bea63023790a", bytes: 509 },
    Remote { url: concat!(base!(), "tiny-streaming-vi/quantized_26_11_07/tokenizer.bin"), sha256: "004723b832413eca43dc95f5c1e14311fa78206f81e6bf99ce0e561c657ab209", bytes: 95_323 },
];
const ZH: [Remote; 8] = [
    Remote { url: concat!(base!(), "tiny-streaming-zh/quantized_26_11_07/adapter.ort"), sha256: "84d1bed88310f5b6dd452203bcaffb765f017a7d06da8d84a898235ef5c90e12", bytes: 1_318_472 },
    Remote { url: concat!(base!(), "tiny-streaming-zh/quantized_26_11_07/cross_kv.ort"), sha256: "22c2261f3120b766a91deceb4f32ca0bef0a8f2b181509ad0dc9fe6adbd0c23a", bytes: 1_288_120 },
    Remote { url: concat!(base!(), "tiny-streaming-zh/quantized_26_11_07/decoder_kv.ort"), sha256: "5caf9e4e9eaa39b93122338bfbad2917630afc51293f6229b935ae7df28bed33", bytes: 19_717_336 },
    Remote { url: concat!(base!(), "tiny-streaming-zh/quantized_26_11_07/encoder.ort"), sha256: "0dbb59314c6009aaa65a183933cb2178b88b814c77973b7275d9070ded3a1dbd", bytes: 7_674_856 },
    Remote { url: concat!(base!(), "tiny-streaming-zh/quantized_26_11_07/frontend.model.ort"), sha256: "d27a9c994b43fa6568121f502e8b99d511ffd0425d4090ffddcc82c9a5667971", bytes: 27_608 },
    Remote { url: concat!(base!(), "tiny-streaming-zh/quantized_26_11_07/frontend.weights.ort"), sha256: "ac56e8ee7780e1f254ee26f2d9c9a3b3c6cfd2f6822cb3944924ae321f86dc50", bytes: 2_090_728 },
    Remote { url: concat!(base!(), "tiny-streaming-zh/quantized_26_11_07/streaming_config.json"), sha256: "d31afd7c2dd5ee9ae3fa232312c5b42cb3e4fe1c6e9c53d885f2bea63023790a", bytes: 509 },
    Remote { url: concat!(base!(), "tiny-streaming-zh/quantized_26_11_07/tokenizer.bin"), sha256: "f172da002a818b9968d5c7a995cc7c51acf2168331ab5f914b5b0860ceab8236", bytes: 74_587 },
];
const JA: [Remote; 8] = [
    Remote { url: concat!(base!(), "small-streaming-ja/quantized_26_11_07/adapter.ort"), sha256: "edcf20b9d0bc0a92bc0ddf37d8b96599de35a6cbee970368ed74c774b055e546", bytes: 2_869_296 },
    Remote { url: concat!(base!(), "small-streaming-ja/quantized_26_11_07/cross_kv.ort"), sha256: "fbd18ec7b9514391d7121f4d4abec1be05a60e94a0f174061428380bf772c3e7", bytes: 5_358_752 },
    Remote { url: concat!(base!(), "small-streaming-ja/quantized_26_11_07/decoder_kv.ort"), sha256: "1246d81eeb96df05d0163dcd48ce271a061b01aa3aea047239fbde22485248be", bytes: 61_314_512 },
    Remote { url: concat!(base!(), "small-streaming-ja/quantized_26_11_07/encoder.ort"), sha256: "526b28674f97f6331712ed44bb0727ce95dc2e0e139f9e70c6f0b7c8aa9c41d5", bytes: 44_144_096 },
    Remote { url: concat!(base!(), "small-streaming-ja/quantized_26_11_07/frontend.model.ort"), sha256: "de65092295f011a39ee569f03d073e3564d94252efd74ebde16d720bcc43e38b", bytes: 31_208 },
    Remote { url: concat!(base!(), "small-streaming-ja/quantized_26_11_07/frontend.weights.ort"), sha256: "4e89578a495a6c4dd5632e7acaf2fc38841e6831738c2702ef10f873125a14ec", bytes: 7_769_288 },
    Remote { url: concat!(base!(), "small-streaming-ja/quantized_26_11_07/streaming_config.json"), sha256: "12d16c7f5ea6734d197b79baf47914ea7e996d6fca8d303a19aca10d0617cecc", bytes: 512 },
    Remote { url: concat!(base!(), "small-streaming-ja/quantized_26_11_07/tokenizer.bin"), sha256: "9f2599f20c3a9b03d79dbe122e443647c6589ce8e5c73e65e5c5d6f73485d5de", bytes: 101_836 },
];
const ES: [Remote; 8] = [
    Remote { url: concat!(base!(), "small-streaming-es/quantized_26_11_07/adapter.ort"), sha256: "04b54114c8aab534222922640f7ca0882948ff9f6ed76777f6e184e55a8e8b15", bytes: 2_869_296 },
    Remote { url: concat!(base!(), "small-streaming-es/quantized_26_11_07/cross_kv.ort"), sha256: "4bfd0a641d72ccdae22751f86bbb2e25ff70c1fba4f81213f2415a1accff9618", bytes: 5_358_752 },
    Remote { url: concat!(base!(), "small-streaming-es/quantized_26_11_07/decoder_kv.ort"), sha256: "5b77c3d6baf801ef925a5bc54d7eb3db0c35ebbb86f8eaf5390d7f5bb42fef37", bytes: 61_314_512 },
    Remote { url: concat!(base!(), "small-streaming-es/quantized_26_11_07/encoder.ort"), sha256: "5fa525788d37493e1974f7293c49d54806bca2598de9fe140de3d05189e03045", bytes: 44_144_096 },
    Remote { url: concat!(base!(), "small-streaming-es/quantized_26_11_07/frontend.model.ort"), sha256: "69c76287f49db365aa278d4908ec450e69ca1b4bfb7e836159d963d623ee0c13", bytes: 26_776 },
    Remote { url: concat!(base!(), "small-streaming-es/quantized_26_11_07/frontend.weights.ort"), sha256: "2f5a0eb5f3004c9447810d74274a352746319a32144e31d95ef410f425b84dda", bytes: 7_769_280 },
    Remote { url: concat!(base!(), "small-streaming-es/quantized_26_11_07/streaming_config.json"), sha256: "12d16c7f5ea6734d197b79baf47914ea7e996d6fca8d303a19aca10d0617cecc", bytes: 512 },
    Remote { url: concat!(base!(), "small-streaming-es/quantized_26_11_07/tokenizer.bin"), sha256: "5fbb7d4314dcb18e03c5f975609e4a4cd572b22b01d2b2accb1b3e6830696f36", bytes: 102_888 },
];
const DE: [Remote; 8] = [
    Remote { url: concat!(base!(), "small-streaming-de/quantized_26_11_07/adapter.ort"), sha256: "be438a3eb12ba3697a9e474d20f899e52d8e27b43a6fc4c4c809d2a099646744", bytes: 2_869_296 },
    Remote { url: concat!(base!(), "small-streaming-de/quantized_26_11_07/cross_kv.ort"), sha256: "0c329cbf001bebbb9e34835e95676b9eb29e010405da46f65af511b8da16620e", bytes: 5_358_752 },
    Remote { url: concat!(base!(), "small-streaming-de/quantized_26_11_07/decoder_kv.ort"), sha256: "7da8ac9a1da2a486158ccde88c756fd876bf7f11ef1c2ba14b80e8edcd30028c", bytes: 61_314_512 },
    Remote { url: concat!(base!(), "small-streaming-de/quantized_26_11_07/encoder.ort"), sha256: "1aafdbef7cd56bb019d9d744654a3c21265e7442b0369d47528542b535829597", bytes: 44_144_096 },
    Remote { url: concat!(base!(), "small-streaming-de/quantized_26_11_07/frontend.model.ort"), sha256: "a93453223d93aa41d21691568cde3a01125fe00710a5f7004a825e1a322d3025", bytes: 26_776 },
    Remote { url: concat!(base!(), "small-streaming-de/quantized_26_11_07/frontend.weights.ort"), sha256: "8b417e2e0ee762e34d7163d24383823bb8bd1d037fb6966c32b3f86a3de4961f", bytes: 7_769_280 },
    Remote { url: concat!(base!(), "small-streaming-de/quantized_26_11_07/streaming_config.json"), sha256: "12d16c7f5ea6734d197b79baf47914ea7e996d6fca8d303a19aca10d0617cecc", bytes: 512 },
    Remote { url: concat!(base!(), "small-streaming-de/quantized_26_11_07/tokenizer.bin"), sha256: "5725ad3771a1339802f8cebeda0fa8f7723f5282b1ee726c2544774d7399b9a7", bytes: 103_319 },
];
const AR: [Remote; 8] = [
    Remote { url: concat!(base!(), "tiny-streaming-ar/quantized_26_11_07/adapter.ort"), sha256: "9c1278a338b4ff268fc5741be01446d56ca9cba9ff5f4a75c5e1b2c2e2739361", bytes: 1_318_472 },
    Remote { url: concat!(base!(), "tiny-streaming-ar/quantized_26_11_07/cross_kv.ort"), sha256: "e317af289233894f49da429d4561150c2f61ca86c1b9ca91c6edd3fb4fdeecb5", bytes: 1_288_120 },
    Remote { url: concat!(base!(), "tiny-streaming-ar/quantized_26_11_07/decoder_kv.ort"), sha256: "5c9a2bf2d9fc543fd4596b3ab4ab8d09f791aac5682eb1379fd8d59e34237db0", bytes: 19_717_336 },
    Remote { url: concat!(base!(), "tiny-streaming-ar/quantized_26_11_07/encoder.ort"), sha256: "071afa0a97117353c3f5f6e9f6dc32b2b03929461ff51f25385d65a35177d940", bytes: 7_674_856 },
    Remote { url: concat!(base!(), "tiny-streaming-ar/quantized_26_11_07/frontend.model.ort"), sha256: "04c552e66efe3f7206d0a69779864f93bf3cfaf2fa595c9ce5101c8e3854a414", bytes: 23_176 },
    Remote { url: concat!(base!(), "tiny-streaming-ar/quantized_26_11_07/frontend.weights.ort"), sha256: "6cc4d70efffe757452996574d11065e7dac6a7ff0a9be3e2fec1d0266b062b0e", bytes: 2_093_280 },
    Remote { url: concat!(base!(), "tiny-streaming-ar/quantized_26_11_07/streaming_config.json"), sha256: "d31afd7c2dd5ee9ae3fa232312c5b42cb3e4fe1c6e9c53d885f2bea63023790a", bytes: 509 },
    Remote { url: concat!(base!(), "tiny-streaming-ar/quantized_26_11_07/tokenizer.bin"), sha256: "3acea39a89cdf44fa11f10bfdd7d05d9015d901999f8c8fa18988f19dcc6b267", bytes: 135_726 },
];
const TL: [Remote; 8] = [
    Remote { url: concat!(base!(), "tiny-streaming-tl/quantized_26_11_07/adapter.ort"), sha256: "a47413f3bab023770a82c3d2426edaac2571d4b6ef11f1de4f5ef04fe8921815", bytes: 1_318_472 },
    Remote { url: concat!(base!(), "tiny-streaming-tl/quantized_26_11_07/cross_kv.ort"), sha256: "d449d3b4969cac1c0c07536c5c339bda1d8fa2768cebbfc81f4487f365a933bd", bytes: 1_288_120 },
    Remote { url: concat!(base!(), "tiny-streaming-tl/quantized_26_11_07/decoder_kv.ort"), sha256: "7c34e8da5f52d98883a5a47ea5951fd33b2cdbde871189e623baf4f3accf96fc", bytes: 19_717_336 },
    Remote { url: concat!(base!(), "tiny-streaming-tl/quantized_26_11_07/encoder.ort"), sha256: "4e38d2715f69d954e133ee0ade7608e3b0960c15f5b690278107b8a42f040ed4", bytes: 7_674_856 },
    Remote { url: concat!(base!(), "tiny-streaming-tl/quantized_26_11_07/frontend.model.ort"), sha256: "a94955c8ef260db0f9cc04c96ab0d35fd59bab8d61acbb1b8b466dad638e0a87", bytes: 27_608 },
    Remote { url: concat!(base!(), "tiny-streaming-tl/quantized_26_11_07/frontend.weights.ort"), sha256: "83a518fdf989752c52338031fd997d23f84ae9c1c45945a30cac91dcaba93518", bytes: 2_093_288 },
    Remote { url: concat!(base!(), "tiny-streaming-tl/quantized_26_11_07/streaming_config.json"), sha256: "d31afd7c2dd5ee9ae3fa232312c5b42cb3e4fe1c6e9c53d885f2bea63023790a", bytes: 509 },
    Remote { url: concat!(base!(), "tiny-streaming-tl/quantized_26_11_07/tokenizer.bin"), sha256: "f1d4f9f04a3943a535cf3ad53329ae465237dd4a11bd38c5a9db47a9164bf892", bytes: 91_356 },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    En,
    Vi,
    Zh,
    Ja,
    Es,
    De,
    Ar,
    Tl,
}

impl Language {
    pub const ALL: [Language; 8] =
        [Language::En, Language::Vi, Language::Zh, Language::Ja, Language::Es, Language::De, Language::Ar, Language::Tl];

    pub fn code(self) -> &'static str {
        match self {
            Language::En => "en",
            Language::Vi => "vi",
            Language::Zh => "zh",
            Language::Ja => "ja",
            Language::Es => "es",
            Language::De => "de",
            Language::Ar => "ar",
            Language::Tl => "tl",
        }
    }

    pub fn parse(code: &str) -> Option<Language> {
        Language::ALL.into_iter().find(|l| l.code() == code)
    }

    fn remotes(self) -> &'static [Remote; 8] {
        match self {
            Language::En => &EN,
            Language::Vi => &VI,
            Language::Zh => &ZH,
            Language::Ja => &JA,
            Language::Es => &ES,
            Language::De => &DE,
            Language::Ar => &AR,
            Language::Tl => &TL,
        }
    }

    pub fn bytes(self) -> u64 {
        self.remotes().iter().map(|r| r.bytes).sum()
    }

    pub fn tokens_per_second(self) -> f32 {
        match self {
            Language::Zh | Language::Ja => DENSE_TOKENS_PER_SECOND,
            _ => LATIN_TOKENS_PER_SECOND,
        }
    }
}

static IN_USE: Mutex<Vec<Language>> = Mutex::new(Vec::new());

pub struct UseGuard(Language);

impl Drop for UseGuard {
    fn drop(&mut self) {
        let mut in_use = IN_USE.lock().unwrap();
        if let Some(at) = in_use.iter().position(|&l| l == self.0) {
            in_use.swap_remove(at);
        }
    }
}

pub fn acquire(language: Language) -> UseGuard {
    IN_USE.lock().unwrap().push(language);
    UseGuard(language)
}

pub struct Download {
    language: Language,
    cancel: Arc<AtomicBool>,
    done: Arc<AtomicU64>,
    total: u64,
}

#[derive(Clone, Serialize)]
pub struct Progress {
    language: Language,
    done: u64,
    total: u64,
}

#[derive(Clone, Serialize)]
struct Done {
    language: Language,
    error: Option<String>,
}

#[derive(Serialize)]
pub struct ModelStatus {
    language: Language,
    ready: bool,
    bytes: u64,
}

#[derive(Serialize)]
pub struct Status {
    models: Vec<ModelStatus>,
    downloading: Option<Progress>,
}

fn store(app: &AppHandle) -> PathBuf {
    app.path().app_local_data_dir().unwrap_or_default().join(STORE_DIR)
}

pub fn model_dir(app: &AppHandle, language: Language) -> PathBuf {
    store(app).join(language.code())
}

pub fn installed(app: &AppHandle) -> bool {
    store(app).exists()
}

pub fn ready(app: &AppHandle, language: Language) -> bool {
    let dir = model_dir(app, language);
    stream::FILES.iter().all(|f| dir.join(f).is_file()) && crate::onnx::runtime_path(app).is_file()
}

fn fetch_model(app: &AppHandle, language: Language, fetch: &mut Fetch) -> Result<(), String> {
    let target = model_dir(app, language);
    fs::create_dir_all(&target).map_err(|e| e.to_string())?;
    fetch::clean(&target);
    crate::onnx::ensure(app, fetch)?;
    for (remote, file) in language.remotes().iter().zip(stream::FILES) {
        fetch.file(remote, &target.join(file))?;
    }
    Ok(())
}

fn run(app: &AppHandle, language: Language, cancel: &AtomicBool, done: &AtomicU64, total: u64) -> Result<(), String> {
    let mut last = Instant::now() - THROTTLE;
    let mut fetch = Fetch {
        agent: fetch::agent(),
        cancel,
        done,
        report: Box::new(|done| {
            if last.elapsed() >= THROTTLE {
                last = Instant::now();
                let _ = app.emit_to("main", events::CAPTIONS_PROGRESS, Progress { language, done, total });
            }
        }),
    };
    let result = fetch_model(app, language, &mut fetch);
    if result.is_err() {
        let _ = fs::remove_dir_all(model_dir(app, language));
    }
    result
}

#[tauri::command]
pub fn captions_status(app: AppHandle, state: tauri::State<State>) -> Status {
    let downloading = state
        .captions_download
        .lock()
        .unwrap()
        .as_ref()
        .map(|d| Progress { language: d.language, done: d.done.load(Ordering::Relaxed), total: d.total });
    let models =
        Language::ALL.into_iter().map(|language| ModelStatus { language, ready: ready(&app, language), bytes: language.bytes() }).collect();
    Status { models, downloading }
}

#[tauri::command]
pub fn captions_download(app: AppHandle, state: tauri::State<State>, language: Language) -> Result<(), String> {
    let mut slot = state.captions_download.lock().unwrap();
    if slot.is_some() {
        return Err("a download is already running".into());
    }
    if ready(&app, language) {
        return Ok(());
    }
    let cancel = Arc::new(AtomicBool::new(false));
    let done = Arc::new(AtomicU64::new(0));
    let total = language.bytes() + crate::onnx::missing_bytes(&app);
    *slot = Some(Download { language, cancel: cancel.clone(), done: done.clone(), total });
    drop(slot);
    std::thread::spawn(move || {
        let result = run(&app, language, &cancel, &done, total);
        let error = match result {
            Err(_) if cancel.load(Ordering::Relaxed) => None,
            Err(e) => Some(e),
            Ok(()) => None,
        };
        app.state::<State>().captions_download.lock().unwrap().take();
        let _ = app.emit_to("main", events::CAPTIONS_DONE, Done { language, error });
        crate::panel::refresh_models(&app);
    });
    Ok(())
}

#[tauri::command]
pub fn captions_cancel(state: tauri::State<State>) {
    if let Some(download) = state.captions_download.lock().unwrap().as_ref() {
        download.cancel.store(true, Ordering::Relaxed);
    }
}

#[tauri::command]
pub fn captions_delete(app: AppHandle, state: tauri::State<State>, language: Language) -> Result<(), String> {
    let others_busy = state.download.lock().unwrap().is_some()
        || state.whisper_download.lock().unwrap().is_some()
        || state.translate_download.lock().unwrap().is_some()
        || state.cutout_download.lock().unwrap().is_some()
        || state.upscale_download.lock().unwrap().is_some();
    let download = state.captions_download.lock().unwrap();
    if download.as_ref().is_some_and(|d| d.language == language) {
        return Err("the model is downloading".into());
    }
    if IN_USE.lock().unwrap().contains(&language) {
        return Err("the model is in use".into());
    }
    let removed = match fs::remove_dir_all(model_dir(&app, language)) {
        Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e.to_string()),
        _ => Ok(()),
    };
    let _ = fs::remove_dir(store(&app));
    let busy = others_busy || download.is_some();
    drop(download);
    crate::onnx::release(&app, busy);
    crate::panel::refresh_models(&app);
    removed
}

#[tauri::command]
pub fn captions_copy(text: String) -> Result<(), String> {
    arboard::Clipboard::new().and_then(|mut c| c.set_text(text)).map_err(|e| e.to_string())
}
