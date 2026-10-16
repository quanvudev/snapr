import { Item, ItemActions, ItemContent, ItemTitle } from "@/components/ui/item";
import { Switch } from "@/components/ui/switch";
import { useSettingsContext } from "@/components/settings-provider";

export function PaddingRow() {
  const { cfg, t, update } = useSettingsContext();
  return (
    <Item variant="muted">
      <ItemContent>
        <ItemTitle>{t.padding}</ItemTitle>
      </ItemContent>
      <ItemActions>
        <Switch
          checked={cfg.padding.enabled}
          onCheckedChange={(enabled) => update({ ...cfg, padding: { ...cfg.padding, enabled } })}
        />
      </ItemActions>
    </Item>
  );
}
