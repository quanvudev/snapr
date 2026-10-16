import { Button } from "@/components/ui/button";
import { Item, ItemActions, ItemContent, ItemTitle } from "@/components/ui/item";
import { useSettingsContext } from "@/components/settings-provider";
import { SLOTS } from "@/lib/hotkeys";

export function HotkeyRow({ i }: { i: number }) {
  const { cfg, t, rows, textActive, recording, setRecording } = useSettingsContext();
  const slot = SLOTS[i];
  const off = slot === "text" && !textActive;
  const row = off ? undefined : rows[i];
  return (
    <Item variant="muted" size="sm">
      <ItemContent>
        <ItemTitle>{t[slot]}</ItemTitle>
      </ItemContent>
      <ItemActions className="gap-3">
        {row &&
          (row.error ? (
            <span className="text-xs text-red-500">{t[row.error]}</span>
          ) : row.in_use ? (
            <span className="text-xs text-orange-500">{t.inUse}</span>
          ) : (
            <span className="text-xs text-green-500">✓</span>
          ))}
        <span className="text-xs text-muted-foreground">{row && `→ ${row.combo}`}</span>
        <Button className="w-24" size="sm" disabled={off} variant="outline" onClick={() => setRecording(i)}>
          {recording === i ? t.press : cfg.hotkeys[slot]}
        </Button>
      </ItemActions>
    </Item>
  );
}
