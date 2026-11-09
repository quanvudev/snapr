use std::path::Path;

const SPACE: &str = "\u{2581}";
const BYTE_PREFIX: &[u8] = b"<0x";

pub struct Tokens {
    pieces: Vec<Vec<u8>>,
}

fn byte_token(piece: &[u8]) -> Option<u8> {
    let hex = piece.strip_prefix(BYTE_PREFIX)?.strip_suffix(b">")?;
    u8::from_str_radix(std::str::from_utf8(hex).ok()?, 16).ok()
}

fn is_special(piece: &[u8]) -> bool {
    piece.len() > 2 && piece.first() == Some(&b'<') && piece.last() == Some(&b'>')
}

impl Tokens {
    pub fn load(path: &Path) -> Result<Tokens, String> {
        Tokens::parse(&std::fs::read(path).map_err(|e| e.to_string())?)
    }

    pub fn parse(data: &[u8]) -> Result<Tokens, String> {
        let mut pieces = Vec::new();
        let mut at = 0;
        while at < data.len() {
            let first = data[at] as usize;
            at += 1;
            let len = match first {
                0 => 0,
                1..=127 => first,
                _ => {
                    let second = *data.get(at).ok_or("truncated tokenizer")? as usize;
                    at += 1;
                    second * 128 + first - 128
                }
            };
            let piece = data.get(at..at + len).ok_or("truncated tokenizer")?;
            pieces.push(piece.to_vec());
            at += len;
        }
        if pieces.is_empty() {
            return Err("empty tokenizer".into());
        }
        Ok(Tokens { pieces })
    }

    pub fn text(&self, ids: &[i64]) -> String {
        let mut bytes = Vec::new();
        for &id in ids {
            let Some(piece) = usize::try_from(id).ok().and_then(|i| self.pieces.get(i)) else { continue };
            match byte_token(piece) {
                Some(byte) => bytes.push(byte),
                None if is_special(piece) => {}
                None => bytes.extend_from_slice(piece),
            }
        }
        String::from_utf8_lossy(&bytes).replace(SPACE, " ").trim().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(piece: &[u8]) -> Vec<u8> {
        let mut out = vec![piece.len() as u8];
        out.extend_from_slice(piece);
        out
    }

    fn sample() -> Tokens {
        let pieces: [&[u8]; 8] =
            [b"<unk>", b"<s>", b"</s>", b"<0xC3>", b"<0xA0>", "\u{2581}hello".as_bytes(), "\u{2581}th".as_bytes(), b"ere"];
        Tokens::parse(&pieces.iter().flat_map(|p| entry(p)).collect::<Vec<u8>>()).unwrap()
    }

    #[test]
    fn decodes_pieces_and_spaces() {
        assert_eq!(sample().text(&[1, 5, 6, 7, 2]), "hello there");
    }

    #[test]
    fn decodes_byte_fallback() {
        assert_eq!(sample().text(&[5, 3, 4]), "hello\u{e0}");
    }

    #[test]
    fn skips_unknown_ids() {
        assert_eq!(sample().text(&[5, 99, -1]), "hello");
    }

    #[test]
    fn reads_two_byte_lengths() {
        let long = vec![b'a'; 200];
        let mut data = vec![(200 % 128 + 128) as u8, (200 / 128) as u8];
        data.extend_from_slice(&long);
        data.push(0);
        let tokens = Tokens::parse(&data).unwrap();
        assert_eq!(tokens.pieces.len(), 2);
        assert_eq!(tokens.pieces[0].len(), 200);
        assert!(tokens.pieces[1].is_empty());
    }

    #[test]
    #[ignore]
    fn loads_model_tokenizer() {
        let Ok(dir) = std::env::var("SNAPR_CAPTIONS_MODEL") else { return };
        let tokens = Tokens::load(&Path::new(&dir).join("tokenizer.bin")).unwrap();
        assert!(tokens.pieces.len() > 1000);
        assert_eq!(tokens.text(&[1, 2]), "");
    }
}
