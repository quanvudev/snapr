import { Button } from "@/components/ui/button";
import { ItemGroup } from "@/components/ui/item";
import { useSettingsContext } from "@/components/settings-provider";
import { SLOTS } from "@/lib/hotkeys";
import { HotkeyRow } from "./hotkey-row";
import { ModifiersRow } from "./modifiers-row";

export function HotkeyRows() {
  return (
    <>
      <ModifiersRow />
      {SLOTS.map((s, i) => (
        <HotkeyRow key={s} i={i} />
      ))}
    </>
  );
}

export function Hotkeys() {
  const { t, reset } = useSettingsContext();
  return (
    <>
      <ItemGroup className="gap-3">
        <HotkeyRows />
      </ItemGroup>
      <Button variant="outline" onClick={reset}>
        {t.reset}
      </Button>
    </>
  );
}
