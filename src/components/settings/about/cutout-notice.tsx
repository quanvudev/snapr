import { Button } from "@/components/ui/button";
import { Item, ItemActions, ItemContent, ItemDescription, ItemTitle } from "@/components/ui/item";
import { useSettingsContext } from "@/components/settings-provider";
import { api } from "@/lib/api";
import { CUTOUT_URL } from "@/lib/cutout";

export function CutoutNotice() {
  const { t, cutout } = useSettingsContext();
  if (!cutout.status.ready) return null;
  return (
    <Item variant="muted">
      <ItemContent>
        <ItemTitle>{t.cutoutPoweredBy}</ItemTitle>
        <ItemDescription>{t.cutoutNotice}</ItemDescription>
      </ItemContent>
      <ItemActions>
        <Button variant="link" onClick={() => api.openUrl(CUTOUT_URL).catch(() => {})}>
          {t.cutoutLicenseRead}
        </Button>
      </ItemActions>
    </Item>
  );
}
