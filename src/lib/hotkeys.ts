export const MODS = ["Ctrl", "Shift", "Alt", "Win"];
export const SLOTS = ["fullscreen", "window", "area", "text", "record", "draw", "ring"] as const;

export const RING_SLOT = SLOTS.indexOf("ring");

const MOUSE_TOKENS: Record<number, string> = { 1: "Mouse3", 3: "Mouse4", 4: "Mouse5" };

export const mouseCombo = (e: MouseEvent) => {
  const token = MOUSE_TOKENS[e.button];
  if (!token) return null;
  const mods = [e.ctrlKey && "Ctrl", e.altKey && "Alt", e.shiftKey && "Shift"].filter(Boolean);
  return [...mods, token].join("+");
};

export const normalize = (code: string) => code.replace(/^Digit(\d)$/, "$1").replace(/^Key([A-Z])$/, "$1");
