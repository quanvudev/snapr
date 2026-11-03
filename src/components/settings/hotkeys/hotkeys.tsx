import { Button } from "@/components/ui/button";
import { ItemGroup } from "@/components/ui/item";
import { useSettingsContext } from "@/components/settings-provider";
import { GROUPS, SLOTS } from "@/lib/hotkeys";
import { SettingsCard } from "../settings-card";
import { HotkeyRow } from "./hotkey-row";
import { ModeRow } from "./mode-row";
import { ModifiersRow } from "./modifiers-row";
import { RingSwitchRow } from "./ring-switch-row";

type RowGroup = (props: { children: React.ReactNode }) => React.ReactNode;

const PlainGroup: RowGroup = ({ children }) => <ItemGroup className="gap-3">{children}</ItemGroup>;

export function HotkeyRows({ Group = PlainGroup }: { Group?: RowGroup }) {
  const { cfg, t, view } = useSettingsContext();
  return (
    <>
      <Group>
        <ModeRow />
        {cfg.hotkeys.mode === "fixed" && <ModifiersRow />}
      </Group>
      {GROUPS.map((g) => (
        <div key={g.title} className="flex flex-col gap-2">
          <h3 className="text-xs font-medium text-muted-foreground">{t[g.title]}</h3>
          <Group>
            {g.title === "groupTools" && view !== "settings" && <RingSwitchRow />}
            {g.slots
              .filter((s) => s !== "ring" || cfg.ring.enabled)
              .map((s) => (
                <HotkeyRow key={s} i={SLOTS.indexOf(s)} />
              ))}
          </Group>
        </div>
      ))}
    </>
  );
}

export function Hotkeys() {
  const { t, restoreHotkeys } = useSettingsContext();
  return (
    <>
      <HotkeyRows Group={SettingsCard} />
      <Button variant="outline" onClick={restoreHotkeys}>
        {t.restoreDefaults}
      </Button>
    </>
  );
}
