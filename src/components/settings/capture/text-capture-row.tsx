import { Button } from "@/components/ui/button";
import { Item, ItemActions, ItemContent, ItemDescription, ItemTitle } from "@/components/ui/item";
import { Switch } from "@/components/ui/switch";
import { useSettingsContext } from "@/components/settings-provider";
import { api } from "@/lib/api";

export function TextCaptureRow() {
  const { cfg, t, ocr, update } = useSettingsContext();
  return (
    <Item variant="muted">
      <ItemContent>
        <ItemTitle>{t.textCapture}</ItemTitle>
        {!ocr && <ItemDescription>{t.noOcr}</ItemDescription>}
      </ItemContent>
      <ItemActions className="gap-3">
        {!ocr && (
          <Button size="sm" variant="outline" onClick={api.openLanguageSettings}>
            {t.openLanguageSettings}
          </Button>
        )}
        <Switch
          checked={cfg.text_capture}
          disabled={!ocr}
          onCheckedChange={(text_capture) => update({ ...cfg, text_capture })}
        />
      </ItemActions>
    </Item>
  );
}
