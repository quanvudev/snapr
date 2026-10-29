import { Item, ItemActions, ItemContent, ItemFooter, ItemTitle } from "@/components/ui/item";
import { Switch } from "@/components/ui/switch";
import { useSettingsContext } from "@/components/settings-provider";
import { SliderField } from "../slider-field";

export function RadiusRow() {
  const { cfg, t, update } = useSettingsContext();
  return (
    <Item variant="muted">
      <ItemContent>
        <ItemTitle>{t.roundCorners}</ItemTitle>
      </ItemContent>
      <ItemActions>
        <Switch
          checked={cfg.padding.round}
          onCheckedChange={(round) => update({ ...cfg, padding: { ...cfg.padding, round } })}
        />
      </ItemActions>
      {cfg.padding.round && (
        <ItemFooter>
          <SliderField
            label={t.radius}
            value={cfg.padding.radius}
            min={1}
            max={128}
            unit="px"
            onCommit={(radius) => update({ ...cfg, padding: { ...cfg.padding, radius } })}
          />
        </ItemFooter>
      )}
    </Item>
  );
}
