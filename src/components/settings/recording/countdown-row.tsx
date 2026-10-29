import { Item, ItemActions, ItemContent, ItemDescription, ItemTitle } from "@/components/ui/item";
import { Switch } from "@/components/ui/switch";
import { useSettingsContext } from "@/components/settings-provider";

export function CountdownRow() {
  const { cfg, t, update } = useSettingsContext();
  return (
    <Item variant="muted">
      <ItemContent>
        <ItemTitle>{t.countdown}</ItemTitle>
        <ItemDescription>{t.countdownHint}</ItemDescription>
      </ItemContent>
      <ItemActions>
        <Switch
          checked={cfg.record_countdown}
          onCheckedChange={(record_countdown) => update({ ...cfg, record_countdown })}
        />
      </ItemActions>
    </Item>
  );
}
