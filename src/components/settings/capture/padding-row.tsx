import { Item, ItemActions, ItemContent, ItemDescription, ItemFooter, ItemTitle } from "@/components/ui/item";
import { Switch } from "@/components/ui/switch";
import { useSettingsContext } from "@/components/settings-provider";
import { SliderField } from "../slider-field";

export function PaddingRow() {
  const { cfg, t, update } = useSettingsContext();
  return (
    <Item variant="muted">
      <ItemContent>
        <ItemTitle>{t.padding}</ItemTitle>
        <ItemDescription>{t.paddingHint}</ItemDescription>
      </ItemContent>
      <ItemActions>
        <Switch
          checked={cfg.padding.enabled}
          onCheckedChange={(enabled) => update({ ...cfg, padding: { ...cfg.padding, enabled } })}
        />
      </ItemActions>
      {cfg.padding.enabled && (
        <ItemFooter>
          <SliderField
            label={t.paddingSize}
            value={cfg.padding.size}
            min={1}
            max={512}
            unit="px"
            onCommit={(size) => update({ ...cfg, padding: { ...cfg.padding, size } })}
          />
        </ItemFooter>
      )}
    </Item>
  );
}
