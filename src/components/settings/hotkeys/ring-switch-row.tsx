import { Item, ItemActions, ItemContent, ItemDescription, ItemTitle } from "@/components/ui/item";
import { Switch } from "@/components/ui/switch";
import { useSettingsContext } from "@/components/settings-provider";

export function RingSwitchRow() {
  const { cfg, t, update } = useSettingsContext();
  return (
    <Item variant="muted" size="sm">
      <ItemContent>
        <ItemTitle>{t.ring}</ItemTitle>
        <ItemDescription>{t.ringSwitchHint}</ItemDescription>
      </ItemContent>
      <ItemActions>
        <Switch
          checked={cfg.ring.enabled}
          onCheckedChange={(enabled) => update({ ...cfg, ring: { ...cfg.ring, enabled } })}
        />
      </ItemActions>
    </Item>
  );
}
