import type { Track } from "./api";

export type Language = { code: string; en: string; vi: string };

export const LANGUAGES: Language[] = [
  { code: "zh", en: "Chinese", vi: "Tiếng Trung (giản thể)" },
  { code: "zh-Hant", en: "Traditional Chinese", vi: "Tiếng Trung (phồn thể)" },
  { code: "yue", en: "Cantonese", vi: "Tiếng Quảng Đông" },
  { code: "en", en: "English", vi: "Tiếng Anh" },
  { code: "fr", en: "French", vi: "Tiếng Pháp" },
  { code: "pt", en: "Portuguese", vi: "Tiếng Bồ Đào Nha" },
  { code: "es", en: "Spanish", vi: "Tiếng Tây Ban Nha" },
  { code: "ja", en: "Japanese", vi: "Tiếng Nhật" },
  { code: "tr", en: "Turkish", vi: "Tiếng Thổ Nhĩ Kỳ" },
  { code: "ru", en: "Russian", vi: "Tiếng Nga" },
  { code: "ar", en: "Arabic", vi: "Tiếng Ả Rập" },
  { code: "ko", en: "Korean", vi: "Tiếng Hàn" },
  { code: "th", en: "Thai", vi: "Tiếng Thái" },
  { code: "it", en: "Italian", vi: "Tiếng Ý" },
  { code: "de", en: "German", vi: "Tiếng Đức" },
  { code: "vi", en: "Vietnamese", vi: "Tiếng Việt" },
  { code: "ms", en: "Malay", vi: "Tiếng Mã Lai" },
  { code: "id", en: "Indonesian", vi: "Tiếng Indonesia" },
  { code: "tl", en: "Filipino", vi: "Tiếng Filipino" },
  { code: "hi", en: "Hindi", vi: "Tiếng Hindi" },
  { code: "pl", en: "Polish", vi: "Tiếng Ba Lan" },
  { code: "cs", en: "Czech", vi: "Tiếng Séc" },
  { code: "nl", en: "Dutch", vi: "Tiếng Hà Lan" },
  { code: "km", en: "Khmer", vi: "Tiếng Khmer" },
  { code: "my", en: "Burmese", vi: "Tiếng Miến Điện" },
  { code: "fa", en: "Persian", vi: "Tiếng Ba Tư" },
  { code: "gu", en: "Gujarati", vi: "Tiếng Gujarat" },
  { code: "ur", en: "Urdu", vi: "Tiếng Urdu" },
  { code: "te", en: "Telugu", vi: "Tiếng Telugu" },
  { code: "mr", en: "Marathi", vi: "Tiếng Marathi" },
  { code: "he", en: "Hebrew", vi: "Tiếng Do Thái" },
  { code: "bn", en: "Bengali", vi: "Tiếng Bengal" },
  { code: "ta", en: "Tamil", vi: "Tiếng Tamil" },
  { code: "uk", en: "Ukrainian", vi: "Tiếng Ukraina" },
  { code: "bo", en: "Tibetan", vi: "Tiếng Tây Tạng" },
  { code: "kk", en: "Kazakh", vi: "Tiếng Kazakh" },
  { code: "mn", en: "Mongolian", vi: "Tiếng Mông Cổ" },
  { code: "ug", en: "Uyghur", vi: "Tiếng Duy Ngô Nhĩ" },
];

export function languageName(code: string, ui: string) {
  const language = LANGUAGES.find((l) => l.code.toLowerCase() === code.toLowerCase());
  if (!language) return code;
  return ui === "vi" ? language.vi : language.en;
}

export function defaultTarget(source: string | null, ui: string) {
  if (source === "vi") return "en";
  if (source === "en") return "vi";
  return ui === "en" ? "en" : "vi";
}

export function targets(source: string | null) {
  return LANGUAGES.filter((l) => l.code.toLowerCase() !== source?.toLowerCase());
}

export function trackName(track: Track, ui: string, original: string, source: string | null = null) {
  if (track !== "original") return languageName(track, ui);
  return source ? `${original} (${languageName(source, ui)})` : original;
}
