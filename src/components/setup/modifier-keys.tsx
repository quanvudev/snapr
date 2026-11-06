import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { useSettingsContext } from "@/components/settings-provider";
import { MODS } from "@/lib/hotkeys";

export function ModifierKeys() {
  const { cfg, update } = useSettingsContext();
  const mods = cfg.hotkeys.modifiers.split("+");
  const change = (value: string[]) =>
    update({ ...cfg, hotkeys: { ...cfg.hotkeys, modifiers: MODS.filter((m) => value.includes(m)).join("+") } });

  return (
    <ToggleGroup multiple spacing={2} value={mods} onValueChange={change}>
      {MODS.map((m) => (
        <ToggleGroupItem
          key={m}
          value={m}
          className="h-10 min-w-16 rounded-lg border border-white/10 bg-white/5 text-sm text-muted-foreground shadow-[inset_0_-2px_0_rgba(255,255,255,0.06)] hover:bg-white/10 aria-pressed:border-white/30 aria-pressed:bg-white/90 aria-pressed:text-neutral-950 aria-pressed:shadow-[inset_0_-2px_0_rgba(0,0,0,0.15)]"
        >
          {m}
        </ToggleGroupItem>
      ))}
    </ToggleGroup>
  );
}
