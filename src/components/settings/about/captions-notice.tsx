import { Button } from "@/components/ui/button";
import { Item, ItemActions, ItemContent, ItemDescription, ItemTitle } from "@/components/ui/item";
import { useSettingsContext } from "@/components/settings-provider";
import { api } from "@/lib/api";
import { CAPTIONS_URL } from "@/lib/captions";

export function CaptionsNotice() {
  const { t, captions } = useSettingsContext();
  if (!captions.status.models.some((m) => m.ready)) return null;
  return (
    <Item variant="muted">
      <ItemContent>
        <ItemTitle>{t.captionsPoweredBy}</ItemTitle>
        <ItemDescription>{t.captionsNotice}</ItemDescription>
      </ItemContent>
      <ItemActions>
        <Button variant="link" onClick={() => api.openUrl(CAPTIONS_URL).catch(() => {})}>
          {t.cutoutLicenseRead}
        </Button>
      </ItemActions>
    </Item>
  );
}
