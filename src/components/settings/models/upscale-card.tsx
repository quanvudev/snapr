import { Button } from "@/components/ui/button";
import { Item, ItemActions, ItemContent, ItemDescription } from "@/components/ui/item";
import { useSettingsContext } from "@/components/settings-provider";
import { api } from "@/lib/api";
import { UPSCALE_LICENSE_NAME, UPSCALE_NAME, UPSCALE_URL } from "@/lib/upscale";
import { SettingsCard } from "../settings-card";
import { ModelCard } from "./model-card";
import { ModelUsageRow } from "./model-usage-row";
import type { ModelStore } from "./stores";

export function UpscaleCard({ store }: { store: ModelStore }) {
  const { t } = useSettingsContext();
  return (
    <SettingsCard>
      <ModelCard title={UPSCALE_NAME} description={t.upscaleDescription} store={store} />
      <Item variant="muted" size="sm">
        <ItemContent>
          <ItemDescription>{t.upscaleLicenseLine.replace("{license}", UPSCALE_LICENSE_NAME)}</ItemDescription>
        </ItemContent>
        <ItemActions>
          <Button variant="link" size="sm" onClick={() => api.openUrl(UPSCALE_URL).catch(() => {})}>
            {t.upscaleLicenseRead}
          </Button>
        </ItemActions>
      </Item>
      <ModelUsageRow text={t.modelUsedUpscale} section="gallery" label={`${t.gallery} →`} />
    </SettingsCard>
  );
}
