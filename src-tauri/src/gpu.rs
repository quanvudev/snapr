use serde::Serialize;
use std::sync::Mutex;
use windows::Win32::Graphics::Dxgi::{CreateDXGIFactory1, DXGI_ADAPTER_FLAG_SOFTWARE, IDXGIFactory1};

const MIN_MEMORY: u64 = 2 << 30;
const NVIDIA: u32 = 0x10DE;

static CACHE: Mutex<Option<Option<Adapter>>> = Mutex::new(None);

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Adapter {
    pub index: u32,
    pub name: String,
    pub vendor: u32,
    pub memory: u64,
}

impl Adapter {
    pub fn nvidia(&self) -> bool {
        self.vendor == NVIDIA
    }
}

pub fn adapters() -> Vec<Adapter> {
    let Ok(factory) = (unsafe { CreateDXGIFactory1::<IDXGIFactory1>() }) else { return Vec::new() };
    let mut found = Vec::new();
    let mut index = 0;
    while let Ok(adapter) = unsafe { factory.EnumAdapters1(index) } {
        if let Ok(desc) = unsafe { adapter.GetDesc1() } {
            let software = desc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32 != 0;
            let memory = desc.DedicatedVideoMemory as u64;
            if !software && memory >= MIN_MEMORY {
                let end = desc.Description.iter().position(|&c| c == 0).unwrap_or(desc.Description.len());
                let name = String::from_utf16_lossy(&desc.Description[..end]).trim().to_owned();
                let adapter = Adapter { index, name, vendor: desc.VendorId, memory };
                if !found.iter().any(|a: &Adapter| a.name == adapter.name && a.vendor == adapter.vendor && a.memory == memory) {
                    found.push(adapter);
                }
            }
        }
        index += 1;
    }
    found
}

pub fn best() -> Option<Adapter> {
    adapters().into_iter().rev().max_by_key(|adapter| adapter.memory)
}

pub fn cached() -> Option<Adapter> {
    CACHE.lock().unwrap().get_or_insert_with(best).clone()
}

pub fn refresh() -> Option<Adapter> {
    let adapter = best();
    *CACHE.lock().unwrap() = Some(adapter.clone());
    adapter
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore]
    fn lists_adapters() {
        for adapter in super::adapters() {
            eprintln!("{adapter:?}");
        }
        eprintln!("best: {:?}", super::best());
    }
}
