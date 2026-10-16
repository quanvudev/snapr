import { Item, ItemActions, ItemContent, ItemTitle } from "@/components/ui/item";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { useSettingsContext } from "@/components/settings-provider";
import { MODS } from "@/lib/hotkeys";

export function ModifiersRow() {
  const { cfg, t, update } = useSettingsContext();
  const mods = cfg.hotkeys.modifiers.split("+");
  const change = (value: string[]) =>
    update({
      ...cfg,
      hotkeys: { ...cfg.hotkeys, modifiers: MODS.filter((m) => value.includes(m)).join("+") },
    });

  return (
    <Item variant="muted" size="sm">
      <ItemContent>
        <ItemTitle>{t.modifiers}</ItemTitle>
      </ItemContent>
      <ItemActions>
        <ToggleGroup multiple variant="outline" size="sm" value={mods} onValueChange={change}>
          {MODS.map((m) => (
            <ToggleGroupItem
              key={m}
              value={m}
              className="text-muted-foreground aria-pressed:border-primary aria-pressed:bg-primary aria-pressed:text-primary-foreground"
            >
              {m}
            </ToggleGroupItem>
          ))}
        </ToggleGroup>
      </ItemActions>
    </Item>
  );
}
