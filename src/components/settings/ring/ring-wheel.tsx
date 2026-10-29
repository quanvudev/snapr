import { AppWindow, Plus, Trash2, X } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "@/components/ui/dropdown-menu";
import { useSettingsContext } from "@/components/settings-provider";
import type { BuiltinId, RingItem } from "@/lib/api";
import {
  BUILTINS,
  MAX_RING_ITEMS,
  RING_BUTTON_RADIUS,
  resolveKeys,
  slotCount,
  slotPosition,
  wheelOrbit,
} from "@/lib/ring";
import { cn } from "@/lib/utils";
import { Glyph, RingIcon } from "./ring-icon";
import type { RingDrag } from "./use-ring-drag";

const BUTTON_SIZE = 2 * RING_BUTTON_RADIUS;
const LABEL_ROOM = 130;
const LABEL_ROOM_VERTICAL = 32;
const LABEL_GAP = 8;
const NUMBER_BADGE_OFFSET = 4;
const LABEL_SIDE_THRESHOLD = 0.3;
const FULL_ORBIT = wheelOrbit(MAX_RING_ITEMS);
const WHEEL_WIDTH = 2 * (FULL_ORBIT + RING_BUTTON_RADIUS + LABEL_ROOM);
const WHEEL_HEIGHT = 2 * (FULL_ORBIT + RING_BUTTON_RADIUS + LABEL_ROOM_VERTICAL);
const CENTRE = { x: WHEEL_WIDTH / 2, y: WHEEL_HEIGHT / 2 };

const labelPlacement = (x: number, y: number, side: number) => {
  const left = CENTRE.x + x;
  const top = CENTRE.y + y;
  if (side > LABEL_SIDE_THRESHOLD) return { left: left + RING_BUTTON_RADIUS + LABEL_GAP, top, transform: "translate(0, -50%)" };
  if (side < -LABEL_SIDE_THRESHOLD) return { left: left - RING_BUTTON_RADIUS - LABEL_GAP, top, transform: "translate(-100%, -50%)" };
  if (y < 0) return { left, top: top - RING_BUTTON_RADIUS - LABEL_GAP, transform: "translate(-50%, -100%)" };
  return { left, top: top + RING_BUTTON_RADIUS + LABEL_GAP, transform: "translate(-50%, 0)" };
};

type Props = {
  items: RingItem[];
  selected: number | null;
  removed: BuiltinId[];
  drag: RingDrag;
  keys: boolean;
  onSelect: (index: number | null) => void;
  onAddBuiltin: (id: BuiltinId) => void;
  onAddApp: () => void;
};

export function RingWheel({ items, selected, removed, drag, keys, onSelect, onAddBuiltin, onAddApp }: Props) {
  const { t } = useSettingsContext();
  const slots = slotCount(items.length);
  const keyLabels = resolveKeys(items);
  const guideSize = 2 * wheelOrbit(slots);
  const buttonPosition = (i: number) => {
    const { x, y } = slotPosition(i, slots);
    return { left: CENTRE.x + x - RING_BUTTON_RADIUS, top: CENTRE.y + y - RING_BUTTON_RADIUS, width: BUTTON_SIZE, height: BUTTON_SIZE };
  };

  return (
    <div className="flex flex-col items-center gap-3">
      <div
        className="relative"
        style={{ width: WHEEL_WIDTH, height: WHEEL_HEIGHT }}
        onClick={(e) => e.target === e.currentTarget && onSelect(null)}
        {...drag.circle}
      >
        <div
          className="pointer-events-none absolute rounded-full border border-border bg-card"
          style={{ left: CENTRE.x - guideSize / 2, top: CENTRE.y - guideSize / 2, width: guideSize, height: guideSize }}
        />
        <X className="pointer-events-none absolute size-4 -translate-x-1/2 -translate-y-1/2 text-muted-foreground/50" style={{ left: CENTRE.x, top: CENTRE.y }} />
        {items.map((item, i) => {
          const name = item.kind === "builtin" ? t[BUILTINS[item.id].name] : item.label;
          const { x, y } = slotPosition(i, slots);
          return (
            <div key={i}>
              <Button
                size="icon"
                variant="outline"
                aria-label={name}
                aria-pressed={selected === i}
                draggable
                onClick={() => onSelect(selected === i ? null : i)}
                onDragStart={drag.start({ kind: "slot", index: i })}
                onDragEnd={drag.end}
                className={cn(
                  "absolute rounded-full bg-secondary shadow-sm hover:bg-accent dark:bg-secondary dark:hover:bg-accent",
                  selected === i && "border-primary ring-2 ring-primary",
                  drag.over === i && "ring-2 ring-primary/60",
                )}
                style={buttonPosition(i)}
              >
                <RingIcon item={item} />
              </Button>
              {keys && keyLabels[i] && (
                <Badge
                  className="pointer-events-none absolute h-4 min-w-4 justify-center px-1 text-[10px]"
                  style={{
                    left: buttonPosition(i).left - NUMBER_BADGE_OFFSET,
                    top: buttonPosition(i).top - NUMBER_BADGE_OFFSET,
                  }}
                >
                  {keyLabels[i]}
                </Badge>
              )}
              <Badge
                variant="secondary"
                className="pointer-events-none absolute truncate"
                style={{ ...labelPlacement(x, y, Math.sin((i * 2 * Math.PI) / slots)), maxWidth: LABEL_ROOM - LABEL_GAP }}
              >
                {name}
              </Badge>
            </div>
          );
        })}
        {items.length < MAX_RING_ITEMS && (
          <DropdownMenu>
            <DropdownMenuTrigger
              render={
                <Button
                  size="icon"
                  variant="outline"
                  aria-label={t.ringAddToRing}
                  className={cn("absolute rounded-full border-dashed bg-card text-muted-foreground hover:text-foreground dark:bg-card", drag.over === items.length && "opacity-100 ring-2 ring-primary/60")}
                  style={buttonPosition(items.length)}
                />
              }
            >
              <Plus />
            </DropdownMenuTrigger>
            <DropdownMenuContent className="w-auto">
              {removed.map((id) => {
                return (
                  <DropdownMenuItem key={id} onClick={() => onAddBuiltin(id)}>
                    <Glyph char={BUILTINS[id].glyph} />
                    {t[BUILTINS[id].name]}
                  </DropdownMenuItem>
                );
              })}
              <DropdownMenuItem onClick={onAddApp}>
                <AppWindow />
                {t.ringAddApp}
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        )}
      </div>
      <div
        className={cn(
          "flex items-center gap-2 rounded-md border border-dashed px-4 py-3 text-sm text-muted-foreground",
          !drag.removing && "invisible",
        )}
        {...drag.bin}
      >
        <Trash2 className="size-4" />
        {t.ringBin}
      </div>
    </div>
  );
}
