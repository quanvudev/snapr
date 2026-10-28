import type { Hotkeys } from "@/lib/api";

export const MODS = ["Ctrl", "Shift", "Alt", "Win"];
export const SLOTS = ["fullscreen", "window", "area", "text", "record", "draw", "ring"] as const;

export const RING_SLOT = SLOTS.indexOf("ring");

export const GROUPS = [
  { title: "groupCapture", slots: ["fullscreen", "window", "area", "text", "record", "draw"] },
  { title: "groupTools", slots: ["ring"] },
] as const;

export const CLEARABLE_SLOTS: readonly string[] = ["text", "draw", "ring"];

const isBareKey = (key: string) => !!key && !key.includes("+") && !/^Mouse\d/.test(key);

export const defaultCombos = (hotkeys: Hotkeys) =>
  SLOTS.map((s) => (isBareKey(hotkeys[s]) ? `${hotkeys.modifiers}+${hotkeys[s]}` : ""));

const MOUSE_TOKENS: Record<number, string> = { 1: "Mouse3", 3: "Mouse4", 4: "Mouse5" };

export const mouseCombo = (e: MouseEvent) => {
  const token = MOUSE_TOKENS[e.button];
  if (!token) return null;
  const mods = [e.ctrlKey && "Ctrl", e.altKey && "Alt", e.shiftKey && "Shift"].filter(Boolean);
  return [...mods, token].join("+");
};

export const normalize = (code: string) => code.replace(/^Digit(\d)$/, "$1").replace(/^Key([A-Z])$/, "$1");

export const keyCombo = (e: KeyboardEvent, key: string) => {
  const held = [e.ctrlKey, e.shiftKey, e.altKey, e.metaKey];
  return [...MODS.filter((_, i) => held[i]), key].join("+");
};
