use std::ops::Range;
use std::sync::OnceLock;

use regex::Regex;

use crate::config;
use crate::ocr::{Bounds, OcrLine, union};

pub const MAX_PATTERNS: usize = 20;
pub const MAX_PATTERN_LEN: usize = 200;
pub const EMPTY: &str = "EMPTY";
pub const TOO_LONG: &str = "TOO_LONG";
const PAD: f32 = 2.0;
const PHONE_DIGITS: Range<usize> = 9..16;
const CARD_DIGITS: Range<usize> = 13..20;
const SECRET_LEN: usize = 32;

struct Builtin {
    email: Regex,
    phone: Regex,
    card: Regex,
    ipv4: Regex,
    secrets: [Regex; 5],
}

fn builtin() -> &'static Builtin {
    static PATTERNS: OnceLock<Builtin> = OnceLock::new();
    PATTERNS.get_or_init(|| {
        let re = |p: &str| Regex::new(p).expect("built-in redaction pattern");
        Builtin {
            email: re(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9-]+(?:\.[A-Za-z0-9-]+)*\.[A-Za-z]{2,}"),
            phone: re(r"(?:\+|\b)\(?\d[\d .()-]{7,}\d\b"),
            card: re(r"\b\d(?:[ -]?\d){12,18}\b"),
            ipv4: re(r"\b(\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3})\b"),
            secrets: [
                re(r"\b(?:sk-|ghp_|gho_|github_pat_|xoxb-|xoxp-)[A-Za-z0-9_-]{8,}"),
                re(r"\bAKIA[0-9A-Z]{16}\b"),
                re(r"\bAIza[0-9A-Za-z_-]{30,}"),
                re(r"\beyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+"),
                re(&format!(r"\b[A-Za-z0-9]{{{SECRET_LEN},}}\b")),
            ],
        }
    })
}

pub struct Rules {
    email: bool,
    phone: bool,
    card: bool,
    ipv4: bool,
    secrets: bool,
    custom: Vec<Regex>,
}

impl Rules {
    pub fn from(cfg: &config::Redact) -> Self {
        let custom = cfg
            .patterns
            .iter()
            .take(MAX_PATTERNS)
            .filter(|p| !p.trim().is_empty() && p.chars().count() <= MAX_PATTERN_LEN)
            .filter_map(|p| Regex::new(p).ok())
            .collect();
        Self { email: cfg.email, phone: cfg.phone, card: cfg.card, ipv4: cfg.ipv4, secrets: cfg.secrets, custom }
    }

    pub fn is_empty(&self) -> bool {
        !(self.email || self.phone || self.card || self.ipv4 || self.secrets) && self.custom.is_empty()
    }

    fn matches(&self, text: &str) -> Vec<Range<usize>> {
        let b = builtin();
        let mut found = Vec::new();
        let ips: Vec<Range<usize>> = b.ipv4.captures_iter(text).filter(|c| is_ipv4(text, c)).filter_map(|c| c.get(0)).map(|m| m.range()).collect();
        if self.email {
            found.extend(b.email.find_iter(text).map(|m| m.range()));
        }
        if self.ipv4 {
            found.extend(ips.iter().cloned());
        }
        if self.card {
            found.extend(b.card.find_iter(text).filter(|m| luhn(m.as_str())).map(|m| m.range()));
        }
        if self.phone {
            found.extend(
                b.phone
                    .find_iter(text)
                    .filter(|m| PHONE_DIGITS.contains(&digits(m.as_str()).len()))
                    .filter(|m| !ips.iter().any(|ip| ip.start <= m.start() && m.end() <= ip.end))
                    .filter(|m| !is_dotted(m.as_str()))
                    .map(|m| m.range()),
            );
        }
        if self.secrets {
            for (i, re) in b.secrets.iter().enumerate() {
                let generic = i == b.secrets.len() - 1;
                found.extend(re.find_iter(text).filter(|m| !generic || mixed(m.as_str())).map(|m| m.range()));
            }
        }
        for re in &self.custom {
            found.extend(re.find_iter(text).filter(|m| !m.is_empty()).map(|m| m.range()));
        }
        merge(found)
    }
}

fn digits(s: &str) -> Vec<u32> {
    s.chars().filter_map(|c| c.to_digit(10)).collect()
}

fn luhn(s: &str) -> bool {
    let d = digits(s);
    if !CARD_DIGITS.contains(&d.len()) {
        return false;
    }
    let sum: u32 = d.iter().rev().enumerate().map(|(i, &n)| if i % 2 == 1 { if n * 2 > 9 { n * 2 - 9 } else { n * 2 } } else { n }).sum();
    sum.is_multiple_of(10)
}

fn is_ipv4(text: &str, c: &regex::Captures) -> bool {
    let Some(whole) = c.get(0) else { return false };
    let octets = (1..=4).all(|i| c.get(i).and_then(|m| m.as_str().parse::<u32>().ok()).is_some_and(|n| n <= 255));
    let before = text[..whole.start()].chars().next_back();
    let mut after = text[whole.end()..].chars();
    let longer = after.next() == Some('.') && after.next().is_some_and(|c| c.is_ascii_digit());
    octets && !matches!(before, Some('v' | 'V' | '.')) && !longer
}

fn is_dotted(s: &str) -> bool {
    s.contains('.') && s.chars().all(|c| c.is_ascii_digit() || c == '.')
}

fn mixed(s: &str) -> bool {
    s.chars().any(|c| c.is_ascii_alphabetic()) && s.chars().any(|c| c.is_ascii_digit())
}

fn merge(mut ranges: Vec<Range<usize>>) -> Vec<Range<usize>> {
    ranges.sort_by_key(|r| r.start);
    let mut out: Vec<Range<usize>> = Vec::new();
    for r in ranges {
        match out.last_mut() {
            Some(last) if r.start <= last.end => last.end = last.end.max(r.end),
            _ => out.push(r),
        }
    }
    out
}

fn cut(text: &str, span: &Range<usize>, rect: Bounds, m: &Range<usize>) -> Option<Bounds> {
    let (start, end) = (m.start.max(span.start), m.end.min(span.end));
    if start >= end {
        return None;
    }
    let chars = |r: Range<usize>| text.get(r).map_or(0, |s| s.chars().count()) as f32;
    let total = chars(span.clone()).max(1.0);
    let left = rect.0 + rect.2 * chars(span.start..start) / total;
    let right = rect.0 + rect.2 * chars(span.start..end) / total;
    Some((left, rect.1, right - left, rect.3))
}

fn locate(line: &OcrLine, m: &Range<usize>) -> Option<Bounds> {
    if line.spans.is_empty() {
        return cut(&line.text, &(0..line.text.len()), line.rect, m);
    }
    line.spans.iter().filter_map(|(span, rect)| cut(&line.text, span, *rect, m)).reduce(union)
}

pub fn find(lines: &[OcrLine], rules: &Rules, (w, h): (u32, u32)) -> Vec<Bounds> {
    let (w, h) = (w as f32, h as f32);
    lines
        .iter()
        .flat_map(|line| rules.matches(&line.text).into_iter().filter_map(move |m| locate(line, &m)))
        .filter_map(|(x, y, rw, rh)| {
            let (x0, y0) = ((x - PAD).max(0.0), (y - PAD).max(0.0));
            let (x1, y1) = ((x + rw + PAD).min(w), (y + rh + PAD).min(h));
            (x1 > x0 && y1 > y0).then_some((x0, y0, x1 - x0, y1 - y0))
        })
        .collect()
}

#[tauri::command]
pub fn redact_validate(pattern: String) -> Result<(), String> {
    if pattern.trim().is_empty() {
        return Err(EMPTY.into());
    }
    if pattern.chars().count() > MAX_PATTERN_LEN {
        return Err(TOO_LONG.into());
    }
    Regex::new(&pattern).map(|_| ()).map_err(|e| e.to_string())
}
