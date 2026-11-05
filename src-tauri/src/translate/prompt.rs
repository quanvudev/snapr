use super::languages::{self, find};

const OTHER: &str = "Translate the following segment into {target}, without additional explanation.\n\n{source}";
const CHINESE: &str = "将以下文本翻译为{target}，注意只需要输出翻译后的结果，不要额外解释：\n\n{source}";

pub fn build(source_lang: &str, target: &str, text: &str) -> String {
    let language = find(target);
    let (template, name) = if languages::is_chinese(source_lang) || languages::is_chinese(target) {
        (CHINESE, language.map_or(target, |l| l.chinese))
    } else {
        (OTHER, language.map_or(target, |l| l.english))
    };
    template.replace("{target}", name).replace("{source}", text.trim())
}

pub fn clean(reply: &str, target: &str) -> String {
    let mut text = reply.trim();
    if let Some(language) = find(target) {
        for prefix in [language.english, language.chinese] {
            if let Some(rest) = text.strip_prefix(prefix)
                && let Some(rest) = rest.trim_start().strip_prefix([':', '：'])
            {
                text = rest.trim();
            }
        }
    }
    for (open, close) in [('"', '"'), ('“', '”'), ('「', '」')] {
        if let Some(inner) = text.strip_prefix(open).and_then(|t| t.strip_suffix(close)) {
            text = inner.trim();
        }
    }
    text.lines().map(str::trim).filter(|l| !l.is_empty()).collect::<Vec<_>>().join(" ")
}
