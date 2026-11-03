import { useEffect, useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { ItemGroup } from "@/components/ui/item";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import { useSettingsContext } from "@/components/settings-provider";
import type { BuiltinId, RingItem } from "@/lib/api";
import { SLOTS } from "@/lib/hotkeys";
import { cn } from "@/lib/utils";
import { insertItem, MAX_RING_ITEMS, moveItem, removedBuiltins, removeItem, ringItems } from "@/lib/ring";
import { HotkeyRow } from "../hotkeys/hotkey-row";
import { RingSwitchRow } from "../hotkeys/ring-switch-row";
import { useAppPicker } from "./app-picker";
import { RingAvailable } from "./ring-available";
import { RingInspector } from "./ring-inspector";
import { RingWheel } from "./ring-wheel";
import { useRingDrag } from "./use-ring-drag";

const IDLE = [0, 3, 5, 10, 30];

export function Ring() {
  const { cfg, t, update } = useSettingsContext();
  const [picked, setPicked] = useState<number | null>(null);
  const [recordingKey, setRecordingKey] = useState(false);
  const items = ringItems(cfg);
  const selected = picked !== null && picked < items.length ? picked : null;
  const full = items.length >= MAX_RING_ITEMS;
  const removed = removedBuiltins(items);
  const save = (next: RingItem[]) => update({ ...cfg, ring: { ...cfg.ring, items: next } });
  const append = (item: RingItem) => {
    save(insertItem(items, item, items.length));
    setPicked(items.length);
  };
  const addBuiltin = (id: BuiltinId) => append({ kind: "builtin", id });
  const picker = useAppPicker();
  const addApp = async () => {
    const item = await picker.pick();
    if (item) append(item);
  };
  const drag = useRingDrag(items, save, setPicked, picker.pick);
  const hasSelection = selected !== null;
  const idleLabel = (s: number) => (s ? `${s} ${t.secondsUnit}` : t.never);

  useEffect(() => {
    if (!hasSelection || recordingKey) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.code !== "Escape") return;
      e.preventDefault();
      setPicked(null);
    };
    window.addEventListener("keydown", onKey, true);
    return () => window.removeEventListener("keydown", onKey, true);
  }, [hasSelection, recordingKey]);

  return (
    <div className="space-y-4" {...drag.outside}>
      <ItemGroup className="gap-3">
        <RingSwitchRow />
        {cfg.ring.enabled && <HotkeyRow i={SLOTS.indexOf("ring")} />}
      </ItemGroup>
      <div className={cn("space-y-4", !cfg.ring.enabled && "pointer-events-none opacity-50")}>
        <div className="flex items-center justify-between gap-2">
          <Badge variant="secondary">
            {items.length} / {MAX_RING_ITEMS} {t.ringEntries}
          </Badge>
          <div className="flex items-center gap-4">
            <div className="flex items-center gap-3">
              <div className="text-right leading-tight">
                <Label>{t.ringIdle}</Label>
                <p className="text-xs text-muted-foreground">{t.ringIdleHint}</p>
              </div>
              <Select
                value={String(cfg.ring.idle_close)}
                onValueChange={(v) => update({ ...cfg, ring: { ...cfg.ring, idle_close: Number(v) } })}
              >
                <SelectTrigger>
                  <SelectValue>{(v: string) => idleLabel(Number(v))}</SelectValue>
                </SelectTrigger>
                <SelectContent>
                  {IDLE.map((s) => (
                    <SelectItem key={s} value={String(s)}>
                      {idleLabel(s)}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="flex items-center gap-3">
              <div className="text-right leading-tight">
                <Label htmlFor="ring-keys">{t.ringKeys}</Label>
                <p className="text-xs text-muted-foreground">{t.ringKeysHint}</p>
              </div>
              <Switch
                id="ring-keys"
                checked={cfg.ring.keys}
                onCheckedChange={(keys) => update({ ...cfg, ring: { ...cfg.ring, keys } })}
              />
            </div>
            <Button
              variant="outline"
              onClick={() => {
                setPicked(null);
                update({ ...cfg, ring: { ...cfg.ring, items: null } });
              }}
            >
              {t.restoreDefaults}
            </Button>
          </div>
        </div>
        <div className="grid grid-cols-[auto_minmax(0,1fr)] items-start gap-6">
          <RingWheel
            items={items}
            selected={selected}
            removed={removed}
            drag={drag}
            keys={cfg.ring.keys}
            onSelect={setPicked}
            onAddBuiltin={addBuiltin}
            onAddApp={addApp}
          />
          <div className="space-y-4">
            {selected === null ? (
              <p className="text-sm text-muted-foreground">{t.ringHint}</p>
            ) : (
              <RingInspector
                item={items[selected]}
                items={items}
                onRecordingKey={setRecordingKey}
                index={selected}
                count={items.length}
                onChange={(next) => save(items.map((x, i) => (i === selected ? next : x)))}
                onMove={(to) => {
                  save(moveItem(items, selected, to));
                  setPicked(to);
                }}
                onRemove={() => {
                  save(removeItem(items, selected));
                  setPicked(null);
                }}
              />
            )}
            <RingAvailable removed={removed} full={full} drag={drag} onAddBuiltin={addBuiltin} onAddApp={addApp} />
          </div>
        </div>
      </div>
      {picker.element}
    </div>
  );
}
