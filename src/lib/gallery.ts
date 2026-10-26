import type { GalleryItem } from "./api";

export type Filter = "all" | "photo" | "video";
export type Period = "day" | "month" | "year";
export type Group = { key: string; date_ms: number; items: GalleryItem[] };

const pad2 = (n: number) => String(n).padStart(2, "0");

export const PERIODS: Record<Period, { key: (d: Date) => string; format: Intl.DateTimeFormatOptions }> = {
  day: {
    key: (d) => `${d.getFullYear()}-${pad2(d.getMonth() + 1)}-${pad2(d.getDate())}`,
    format: { dateStyle: "full" },
  },
  month: {
    key: (d) => `${d.getFullYear()}-${pad2(d.getMonth() + 1)}`,
    format: { year: "numeric", month: "long" },
  },
  year: {
    key: (d) => String(d.getFullYear()),
    format: { year: "numeric" },
  },
};

export function matches(item: GalleryItem, filter: Filter) {
  return filter === "all" || item.kind === filter;
}

export function isViewable(item: GalleryItem) {
  return item.kind !== "other";
}

export function groupBy(items: GalleryItem[], period: Period) {
  const groups = new Map<string, Group>();
  for (const item of items) {
    const key = PERIODS[period].key(new Date(item.date_ms));
    const group = groups.get(key);
    if (group) group.items.push(item);
    else groups.set(key, { key, date_ms: item.date_ms, items: [item] });
  }
  return [...groups.values()];
}

export function formatGroup(ms: number, period: Period, language: string) {
  return new Intl.DateTimeFormat(language, PERIODS[period].format).format(ms);
}

export function fileName(path: string) {
  return path.split(/[\\/]/).pop() ?? path;
}
