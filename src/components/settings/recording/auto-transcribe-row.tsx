import { Item, ItemActions, ItemContent, ItemDescription, ItemTitle } from "@/components/ui/item";
import { Switch } from "@/components/ui/switch";
import { useSettingsContext } from "@/components/settings-provider";

export function AutoTranscribeRow() {
  const { cfg, t, update } = useSettingsContext();
  return (
    <Item variant="muted">
      <ItemContent>
        <ItemTitle>{t.autoTranscribe}</ItemTitle>
        <ItemDescription>{t.autoTranscribeHint}</ItemDescription>
      </ItemContent>
      <ItemActions>
        <Switch
          checked={cfg.record_transcribe}
          onCheckedChange={(record_transcribe) => update({ ...cfg, record_transcribe })}
        />
      </ItemActions>
    </Item>
  );
}
