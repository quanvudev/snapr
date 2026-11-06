import { Button } from "@/components/ui/button";
import { Item, ItemActions, ItemContent, ItemDescription } from "@/components/ui/item";
import { useSettingsContext } from "@/components/settings-provider";
import { api } from "@/lib/api";
import { CUTOUT_LICENSE_NAME, CUTOUT_NAME, CUTOUT_URL } from "@/lib/cutout";
import { SettingsCard } from "../settings-card";
import { ModelCard } from "./model-card";
import { ModelUsageRow } from "./model-usage-row";
import type { ModelStore } from "./stores";

export function CutoutCard({ store }: { store: ModelStore }) {
  const { t } = useSettingsContext();
  return (
    <SettingsCard>
      <ModelCard title={CUTOUT_NAME} description={t.cutoutDescription} store={store} />
      <Item variant="muted" size="sm">
        <ItemContent>
          <ItemDescription>{t.cutoutLicenseLine.replace("{license}", CUTOUT_LICENSE_NAME)}</ItemDescription>
        </ItemContent>
        <ItemActions>
          <Button variant="link" size="sm" onClick={() => api.openUrl(CUTOUT_URL).catch(() => {})}>
            {t.cutoutLicenseRead}
          </Button>
        </ItemActions>
      </Item>
      <ModelUsageRow text={t.modelUsedCutout} section="gallery" label={`${t.gallery} →`} />
    </SettingsCard>
  );
}
