import { Item, ItemActions, ItemContent, ItemTitle } from "@/components/ui/item";
import { Switch } from "@/components/ui/switch";
import { useSettingsContext } from "@/components/settings-provider";

export function ClipboardRow() {
  const { cfg, t, update } = useSettingsContext();
  return (
    <Item variant="muted">
      <ItemContent>
        <ItemTitle>{t.clipboard}</ItemTitle>
      </ItemContent>
      <ItemActions>
        <Switch checked={cfg.copy_to_clipboard} onCheckedChange={(v) => update({ ...cfg, copy_to_clipboard: v })} />
      </ItemActions>
    </Item>
  );
}
