use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

pub(crate) struct Remote {
    pub(crate) url: &'static str,
    pub(crate) sha256: &'static str,
    pub(crate) bytes: u64,
}

pub(crate) fn disk_bytes(path: &Path) -> u64 {
    fs::read_dir(path)
        .map(|entries| entries.flatten().filter_map(|e| e.metadata().ok()).filter(|m| m.is_file()).map(|m| m.len()).sum())
        .unwrap_or(0)
}

pub(crate) fn clean(path: &Path) {
    let Ok(entries) = fs::read_dir(path) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "part") {
            let _ = fs::remove_file(path);
        }
    }
}

pub(crate) fn part(path: &Path) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".part");
    path.with_file_name(name)
}

pub(crate) fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_connect(Some(Duration::from_secs(30)))
        .timeout_per_read(Some(Duration::from_secs(30)))
        .build()
        .into()
}

pub(crate) struct Fetch<'a> {
    pub(crate) agent: ureq::Agent,
    pub(crate) cancel: &'a AtomicBool,
    pub(crate) done: &'a AtomicU64,
    pub(crate) report: Box<dyn FnMut(u64) + 'a>,
}

impl Fetch<'_> {
    pub(crate) fn get(&mut self, remote: &Remote, to: &Path) -> Result<(), String> {
        let result = self.stream(remote, to);
        if result.is_err() {
            let _ = fs::remove_file(to);
        }
        result
    }

    fn stream(&mut self, remote: &Remote, to: &Path) -> Result<(), String> {
        let response = self.agent.get(remote.url).call().map_err(|e| e.to_string())?;
        let mut reader = response.into_body().into_with_config().limit(remote.bytes + 1).reader();
        let mut file = File::create(to).map_err(|e| e.to_string())?;
        let mut hasher = Sha256::new();
        let mut buf = vec![0u8; 64 * 1024];
        let mut received = 0u64;
        loop {
            if self.cancel.load(Ordering::Relaxed) {
                return Err("cancelled".into());
            }
            let n = reader.read(&mut buf).map_err(|e| e.to_string())?;
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
            file.write_all(&buf[..n]).map_err(|e| e.to_string())?;
            received += n as u64;
            let done = self.done.fetch_add(n as u64, Ordering::Relaxed) + n as u64;
            (self.report)(done);
        }
        file.flush().map_err(|e| e.to_string())?;
        drop(file);
        let digest: String = hasher.finalize().iter().map(|b| format!("{b:02x}")).collect();
        if received != remote.bytes || digest != remote.sha256 {
            return Err(format!("checksum mismatch: {}", remote.url.rsplit('/').next().unwrap_or_default()));
        }
        Ok(())
    }

    pub(crate) fn file(&mut self, remote: &Remote, to: &Path) -> Result<(), String> {
        let tmp = part(to);
        self.get(remote, &tmp)?;
        fs::rename(&tmp, to).map_err(|e| {
            let _ = fs::remove_file(&tmp);
            e.to_string()
        })
    }
}
