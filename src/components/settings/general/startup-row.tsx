import { Item, ItemActions, ItemContent, ItemTitle } from "@/components/ui/item";
import { Switch } from "@/components/ui/switch";
import { useSettingsContext } from "@/components/settings-provider";

export function StartupRow() {
  const { cfg, t, autostart, update } = useSettingsContext();
  return (
    <Item variant="muted">
      <ItemContent>
        <ItemTitle>{t.startup}</ItemTitle>
      </ItemContent>
      <ItemActions>
        <Switch checked={autostart} onCheckedChange={(v) => update(cfg, v)} />
      </ItemActions>
    </Item>
  );
}
