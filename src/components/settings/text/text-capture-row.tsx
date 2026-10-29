import { Button } from "@/components/ui/button";
import { Item, ItemActions, ItemContent, ItemDescription, ItemTitle } from "@/components/ui/item";
import { Switch } from "@/components/ui/switch";
import { useSettingsContext } from "@/components/settings-provider";
import { api } from "@/lib/api";

export function TextCaptureRow() {
  const { cfg, t, ocr, windowsOcr, update } = useSettingsContext();
  return (
    <Item variant="muted">
      <ItemContent>
        <ItemTitle>{t.textCapture}</ItemTitle>
        {!ocr && !windowsOcr && <ItemDescription>{t.noOcr}</ItemDescription>}
      </ItemContent>
      <ItemActions className="gap-3">
        {!ocr && !windowsOcr && (
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
