import { Button } from "@/components/ui/button";
import { Item, ItemActions, ItemContent, ItemDescription, ItemTitle } from "@/components/ui/item";
import { useSettingsContext } from "@/components/settings-provider";
import { api } from "@/lib/api";
import { UPSCALE_URL } from "@/lib/upscale";

export function UpscaleNotice() {
  const { t, upscale } = useSettingsContext();
  if (!upscale.status.ready) return null;
  return (
    <Item variant="muted">
      <ItemContent>
        <ItemTitle>{t.upscalePoweredBy}</ItemTitle>
        <ItemDescription>{t.upscaleNotice}</ItemDescription>
      </ItemContent>
      <ItemActions>
        <Button variant="link" onClick={() => api.openUrl(UPSCALE_URL).catch(() => {})}>
          {t.upscaleLicenseRead}
        </Button>
      </ItemActions>
    </Item>
  );
}
