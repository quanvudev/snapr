import { XIcon } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Item, ItemActions, ItemContent, ItemTitle } from "@/components/ui/item";
import { Kbd, KbdGroup } from "@/components/ui/kbd";
import { useSettingsContext } from "@/components/settings-provider";
import { CLEARABLE_SLOTS, SLOTS } from "@/lib/hotkeys";
import { cn } from "@/lib/utils";

export function HotkeyRow({ i }: { i: number }) {
  const { cfg, t, rows, textActive, recording, setRecording, update } = useSettingsContext();
  const slot = SLOTS[i];
  const { hotkeys } = cfg;
  const off = slot === "text" && !textActive;
  const row = off ? undefined : rows[i];
  const stored = hotkeys.mode === "fixed" ? hotkeys[slot] : hotkeys.manual[i];
  const combo = row?.combo ?? stored;
  const isRecording = recording === i;
  const clear = () =>
    update({
      ...cfg,
      hotkeys:
        hotkeys.mode === "fixed"
          ? { ...hotkeys, [slot]: "" }
          : { ...hotkeys, manual: hotkeys.manual.map((c, n) => (n === i ? "" : c)) },
    });

  return (
    <Item
      variant="muted"
      size="sm"
      className={cn(isRecording && "ring-2 ring-primary", recording !== null && !isRecording && "opacity-50")}
    >
      <ItemContent>
        <ItemTitle>{t[slot]}</ItemTitle>
        {row?.error ? (
          <Badge variant="destructive">{t[row.error]}</Badge>
        ) : row?.in_use ? (
          <Badge variant="outline">{t.inUse}</Badge>
        ) : null}
      </ItemContent>
      <ItemActions>
        <Button size="sm" disabled={off} variant="ghost" onClick={() => setRecording(i)}>
          {isRecording ? (
            t.press
          ) : combo ? (
            <KbdGroup>
              {combo.split("+").map((k, n) => (
                <Kbd key={n}>{k}</Kbd>
              ))}
            </KbdGroup>
          ) : (
            <span className="text-muted-foreground">{t.notSet}</span>
          )}
        </Button>
        {CLEARABLE_SLOTS.includes(slot) && combo && !isRecording && (
          <Button size="icon-sm" variant="ghost" aria-label={t.clearHotkey} onClick={clear}>
            <XIcon />
          </Button>
        )}
      </ItemActions>
    </Item>
  );
}
