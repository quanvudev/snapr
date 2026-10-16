export const MODS = ["Ctrl", "Shift", "Alt", "Win"];
export const SLOTS = ["fullscreen", "window", "area", "text", "record"] as const;

export const normalize = (code: string) => code.replace(/^Digit(\d)$/, "$1").replace(/^Key([A-Z])$/, "$1");
