import { Button } from "@/components/ui/button";
import { ItemGroup } from "@/components/ui/item";
import { useSettingsContext } from "@/components/settings-provider";
import { GROUPS, SLOTS } from "@/lib/hotkeys";
import { HotkeyRow } from "./hotkey-row";
import { ModeRow } from "./mode-row";
import { ModifiersRow } from "./modifiers-row";
import { RingSwitchRow } from "./ring-switch-row";

export function HotkeyRows() {
  const { cfg, t } = useSettingsContext();
  return (
    <>
      <ModeRow />
      {cfg.hotkeys.mode === "fixed" && <ModifiersRow />}
      {GROUPS.map((g) => (
        <div key={g.title} className="flex flex-col gap-2">
          <h3 className="text-xs font-medium text-muted-foreground">{t[g.title]}</h3>
          <ItemGroup className="gap-3">
            {g.title === "groupTools" && <RingSwitchRow />}
            {g.slots
              .filter((s) => s !== "ring" || cfg.ring.enabled)
              .map((s) => (
                <HotkeyRow key={s} i={SLOTS.indexOf(s)} />
              ))}
          </ItemGroup>
        </div>
      ))}
    </>
  );
}

export function Hotkeys() {
  const { t, restoreHotkeys } = useSettingsContext();
  return (
    <>
      <ItemGroup className="gap-3">
        <HotkeyRows />
      </ItemGroup>
      <Button variant="outline" onClick={restoreHotkeys}>
        {t.restoreDefaults}
      </Button>
    </>
  );
}
