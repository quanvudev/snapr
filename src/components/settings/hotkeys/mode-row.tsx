import { Item, ItemActions, ItemContent, ItemDescription, ItemTitle } from "@/components/ui/item";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { useSettingsContext } from "@/components/settings-provider";
import type { HotkeyMode } from "@/lib/api";
import { SLOTS } from "@/lib/hotkeys";

const MODES: HotkeyMode[] = ["fixed", "manual"];

export function ModeRow() {
  const { cfg, t, rows, update } = useSettingsContext();
  const change = (value: string[]) => {
    const mode = value[0] as HotkeyMode | undefined;
    if (!mode) return;
    const unseeded = cfg.hotkeys.manual.length < SLOTS.length || cfg.hotkeys.manual.every((c) => !c);
    const manual = mode === "manual" && unseeded ? rows.map((r) => r.combo) : cfg.hotkeys.manual;
    update({ ...cfg, hotkeys: { ...cfg.hotkeys, mode, manual } });
  };

  return (
    <Item variant="muted" size="sm">
      <ItemContent>
        <ItemTitle>{t.hotkeyMode}</ItemTitle>
        <ItemDescription>{cfg.hotkeys.mode === "fixed" ? t.fixedHint : t.manualHint}</ItemDescription>
      </ItemContent>
      <ItemActions>
        <ToggleGroup variant="outline" size="sm" value={[cfg.hotkeys.mode]} onValueChange={change}>
          {MODES.map((m) => (
            <ToggleGroupItem
              key={m}
              value={m}
              className="text-muted-foreground aria-pressed:border-primary aria-pressed:bg-primary aria-pressed:text-primary-foreground"
            >
              {t[m]}
            </ToggleGroupItem>
          ))}
        </ToggleGroup>
      </ItemActions>
    </Item>
  );
}
