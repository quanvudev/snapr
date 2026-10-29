import { Item, ItemActions, ItemContent, ItemFooter, ItemTitle } from "@/components/ui/item";
import { Switch } from "@/components/ui/switch";
import { useSettingsContext } from "@/components/settings-provider";
import { SliderField } from "../slider-field";

export function PreviewRow() {
  const { cfg, t, update } = useSettingsContext();
  const on = cfg.preview_seconds > 0;
  return (
    <Item variant="muted">
      <ItemContent>
        <ItemTitle>{t.previewToast}</ItemTitle>
      </ItemContent>
      <ItemActions>
        <Switch checked={on} onCheckedChange={(v) => update({ ...cfg, preview_seconds: v ? 5 : 0 })} />
      </ItemActions>
      {on && (
        <ItemFooter>
          <SliderField
            label={t.hideAfter}
            value={cfg.preview_seconds}
            min={1}
            max={30}
            unit={t.secondsUnit}
            onCommit={(preview_seconds) => update({ ...cfg, preview_seconds })}
          />
        </ItemFooter>
      )}
    </Item>
  );
}
