import { Item, ItemActions, ItemContent, ItemTitle } from "@/components/ui/item";
import { Switch } from "@/components/ui/switch";
import { useSettingsContext } from "@/components/settings-provider";

export function UpdateCheckRow() {
  const { cfg, t, update } = useSettingsContext();
  return (
    <Item variant="muted">
      <ItemContent>
        <ItemTitle>{t.updateCheck}</ItemTitle>
      </ItemContent>
      <ItemActions>
        <Switch checked={cfg.update_check} onCheckedChange={(v) => update({ ...cfg, update_check: v })} />
      </ItemActions>
    </Item>
  );
}
