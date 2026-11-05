pub struct Language {
    pub code: &'static str,
    pub english: &'static str,
    pub chinese: &'static str,
}

const fn lang(code: &'static str, english: &'static str, chinese: &'static str) -> Language {
    Language { code, english, chinese }
}

pub const LANGUAGES: [Language; 38] = [
    lang("zh", "Chinese", "中文"),
    lang("zh-Hant", "Traditional Chinese", "繁体中文"),
    lang("yue", "Cantonese", "粤语"),
    lang("en", "English", "英语"),
    lang("fr", "French", "法语"),
    lang("pt", "Portuguese", "葡萄牙语"),
    lang("es", "Spanish", "西班牙语"),
    lang("ja", "Japanese", "日语"),
    lang("tr", "Turkish", "土耳其语"),
    lang("ru", "Russian", "俄语"),
    lang("ar", "Arabic", "阿拉伯语"),
    lang("ko", "Korean", "韩语"),
    lang("th", "Thai", "泰语"),
    lang("it", "Italian", "意大利语"),
    lang("de", "German", "德语"),
    lang("vi", "Vietnamese", "越南语"),
    lang("ms", "Malay", "马来语"),
    lang("id", "Indonesian", "印尼语"),
    lang("tl", "Filipino", "菲律宾语"),
    lang("hi", "Hindi", "印地语"),
    lang("pl", "Polish", "波兰语"),
    lang("cs", "Czech", "捷克语"),
    lang("nl", "Dutch", "荷兰语"),
    lang("km", "Khmer", "高棉语"),
    lang("my", "Burmese", "缅甸语"),
    lang("fa", "Persian", "波斯语"),
    lang("gu", "Gujarati", "古吉拉特语"),
    lang("ur", "Urdu", "乌尔都语"),
    lang("te", "Telugu", "泰卢固语"),
    lang("mr", "Marathi", "马拉地语"),
    lang("he", "Hebrew", "希伯来语"),
    lang("bn", "Bengali", "孟加拉语"),
    lang("ta", "Tamil", "泰米尔语"),
    lang("uk", "Ukrainian", "乌克兰语"),
    lang("bo", "Tibetan", "藏语"),
    lang("kk", "Kazakh", "哈萨克语"),
    lang("mn", "Mongolian", "蒙古语"),
    lang("ug", "Uyghur", "维吾尔语"),
];

const ENGLISH: &str = "en";
const VIETNAMESE: &str = "vi";

pub fn find(code: &str) -> Option<&'static Language> {
    LANGUAGES.iter().find(|l| l.code.eq_ignore_ascii_case(code))
}

pub fn is_chinese(code: &str) -> bool {
    matches!(code, "zh" | "zh-Hant" | "yue")
}

pub fn default_target(source: &str, ui: &str) -> &'static str {
    match source {
        VIETNAMESE => ENGLISH,
        ENGLISH => VIETNAMESE,
        _ if ui == ENGLISH => ENGLISH,
        _ => VIETNAMESE,
    }
}
