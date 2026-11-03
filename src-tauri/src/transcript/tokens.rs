use std::collections::HashMap;
use std::path::Path;

const BYTE_COUNT: u32 = 256;
const PRINTABLE: [std::ops::RangeInclusive<u32>; 3] = [0x21..=0x7E, 0xA1..=0xAC, 0xAE..=0xFF];

pub struct Tokens {
    bytes: Vec<Option<Vec<u8>>>,
}

fn unicode_to_byte() -> HashMap<char, u8> {
    let mut map = HashMap::new();
    let mut shifted = 0;
    for b in 0..BYTE_COUNT {
        let code = if PRINTABLE.iter().any(|r| r.contains(&b)) {
            b
        } else {
            shifted += 1;
            BYTE_COUNT + shifted - 1
        };
        if let Some(c) = char::from_u32(code) {
            map.insert(c, b as u8);
        }
    }
    map
}

impl Tokens {
    pub fn load(vocab: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(vocab).map_err(|e| e.to_string())?;
        let vocab: HashMap<String, usize> = serde_json::from_str(&text).map_err(|e| e.to_string())?;
        let table = unicode_to_byte();
        let mut bytes = vec![None; vocab.values().max().map_or(0, |&m| m + 1)];
        for (token, id) in vocab {
            bytes[id] = token.chars().map(|c| table.get(&c).copied()).collect();
        }
        Ok(Self { bytes })
    }

    pub fn decode(&self, ids: &[i64]) -> String {
        let bytes: Vec<u8> = ids
            .iter()
            .filter_map(|&id| usize::try_from(id).ok().and_then(|i| self.bytes.get(i)).and_then(Option::as_ref))
            .flatten()
            .copied()
            .collect();
        String::from_utf8_lossy(&bytes).into_owned()
    }
}
