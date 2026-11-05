import { Button } from "@/components/ui/button";
import { Item, ItemActions, ItemContent, ItemDescription, ItemTitle } from "@/components/ui/item";
import { useSettingsContext } from "@/components/settings-provider";
import { api } from "@/lib/api";
import { HYMT_LICENSE_URL, HYMT_NOTICE } from "@/lib/hymt";

export function HymtNotice() {
  const { t, translator } = useSettingsContext();
  if (!translator.status.ready) return null;
  return (
    <Item variant="muted">
      <ItemContent>
        <ItemTitle>{t.hymtPoweredBy}</ItemTitle>
        <ItemDescription>{HYMT_NOTICE}</ItemDescription>
        <ItemDescription>{t.hymtNotAffiliated}</ItemDescription>
      </ItemContent>
      <ItemActions>
        <Button variant="link" onClick={() => api.openUrl(HYMT_LICENSE_URL).catch(() => {})}>
          {t.hymtLicenseRead}
        </Button>
      </ItemActions>
    </Item>
  );
}
