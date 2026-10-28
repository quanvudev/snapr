import {
  AppWindow,
  Circle,
  Images,
  LayoutPanelTop,
  Maximize,
  Pencil,
  ScanText,
  Settings,
  SquareDashed,
  type LucideIcon,
} from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog";
import type { BuiltinId, Config, InstalledApp, RingItem } from "./api";
import type { Strings } from "./i18n";

export const MAX_RING_ITEMS = 12;

export const BUILTINS: Record<BuiltinId, { icon: LucideIcon; name: keyof Strings }> = {
  fullscreen: { icon: Maximize, name: "ringFullscreen" },
  window: { icon: LayoutPanelTop, name: "ringWindow" },
  area: { icon: SquareDashed, name: "ringArea" },
  text: { icon: ScanText, name: "ringText" },
  record: { icon: Circle, name: "ringRecord" },
  draw: { icon: Pencil, name: "ringDraw" },
  gallery: { icon: Images, name: "ringGallery" },
  settings: { icon: Settings, name: "ringSettings" },
};

const POSITION_KEY_COUNT = 9;
const KEY_CODE = /^(?:Key|Digit|Numpad)([A-Z0-9])$/;
const STORE_APP_PREFIX = "shell:appsfolder\\";
const EXE_FILTER = (name: string) => ({ name, extensions: ["exe"] });

export const GENERIC_APP_ICON = AppWindow;

const DEFAULT_RING: RingItem[] = (Object.keys(BUILTINS) as BuiltinId[]).map((id) => ({ kind: "builtin", id }));

export const ringItems = (cfg: Config): RingItem[] => cfg.ring.items ?? DEFAULT_RING;

export const removedBuiltins = (items: RingItem[]): BuiltinId[] =>
  DEFAULT_RING.flatMap((d) =>
    d.kind === "builtin" && !items.some((i) => i.kind === "builtin" && i.id === d.id) ? [d.id] : [],
  );

const fileStem = (path: string) =>
  (path.split(/[\\/]/).pop() ?? "").replace(/\.[^.]+$/, "");

const appItem = (path: string): RingItem => ({ kind: "app", label: fileStem(path), path, args: "", admin: false });

export const installedAppItem = (app: InstalledApp): RingItem => ({ kind: "app", label: app.name, path: app.path, args: "", admin: false });

export const isStoreApp = (path: string) => path.toLowerCase().startsWith(STORE_APP_PREFIX);

export const itemName = (item: RingItem, t: Strings) => (item.kind === "builtin" ? t[BUILTINS[item.id].name] : item.label);

export const resolveKeys = (items: RingItem[]): (string | null)[] => {
  const taken = new Set<string>();
  return items.map((item, i) => {
    const key = item.key || (i < POSITION_KEY_COUNT ? String(i + 1) : null);
    if (!key || taken.has(key)) return null;
    taken.add(key);
    return key;
  });
};

export const normaliseKey = (code: string): string | null => code.match(KEY_CODE)?.[1] ?? null;

export const pickApp = async (filterName: string): Promise<RingItem | null> => {
  const picked = await open({ filters: [EXE_FILTER(filterName)] });
  return typeof picked === "string" ? appItem(picked) : null;
};

export const insertItem = (items: RingItem[], item: RingItem, at: number): RingItem[] => [
  ...items.slice(0, at),
  item,
  ...items.slice(at),
];

export const removeItem = (items: RingItem[], at: number): RingItem[] => items.filter((_, i) => i !== at);

export const moveItem = (items: RingItem[], from: number, to: number): RingItem[] => {
  if (from === to) return items;
  return insertItem(removeItem(items, from), items[from], to);
};

const RING_ORBIT = 90;
export const RING_BUTTON_RADIUS = 24;
const RING_BUTTON_GAP = 20;
const MIN_SLOT_COUNT = 2;
const FULL_TURN = 2 * Math.PI;

export const ringOrbit = (count: number) =>
  Math.max(
    RING_ORBIT,
    (2 * RING_BUTTON_RADIUS + RING_BUTTON_GAP) / (2 * Math.sin(Math.PI / Math.max(count, MIN_SLOT_COUNT))),
  );

const WHEEL_SCALE = 1.45;

export const wheelOrbit = (count: number) => ringOrbit(count) * WHEEL_SCALE;

export const slotPosition = (index: number, count: number) => {
  const angle = (index * FULL_TURN) / count;
  const orbit = wheelOrbit(count);
  return { x: orbit * Math.sin(angle), y: -orbit * Math.cos(angle) };
};

export const nearestSlot = (dx: number, dy: number, count: number) => {
  if (count === 0) return 0;
  const angle = (Math.atan2(dx, -dy) + FULL_TURN) % FULL_TURN;
  return Math.round(angle / (FULL_TURN / count)) % count;
};

const DROP_MARGIN = 16;

export const slotCount = (entries: number) => (entries < MAX_RING_ITEMS ? entries + 1 : entries);

export const wheelRadius = (entries: number) => wheelOrbit(slotCount(entries)) + RING_BUTTON_RADIUS + DROP_MARGIN;
