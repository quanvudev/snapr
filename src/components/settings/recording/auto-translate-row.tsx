import { Item, ItemActions, ItemContent, ItemDescription, ItemTitle } from "@/components/ui/item";
import { Switch } from "@/components/ui/switch";
import { useSettingsContext } from "@/components/settings-provider";

export function AutoTranslateRow() {
  const { cfg, t, translator, update } = useSettingsContext();
  return (
    <Item variant="muted">
      <ItemContent>
        <ItemTitle>{t.autoTranslate}</ItemTitle>
        <ItemDescription>{t.autoTranslateHint}</ItemDescription>
        {cfg.whisper_model === "turbo" && !translator.status.ready && <ItemDescription>{t.turboTranslateNote}</ItemDescription>}
      </ItemContent>
      <ItemActions>
        <Switch
          checked={cfg.record_translate}
          onCheckedChange={(record_translate) => update({ ...cfg, record_translate })}
        />
      </ItemActions>
    </Item>
  );
}
