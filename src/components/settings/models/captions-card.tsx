import { Button } from "@/components/ui/button";
import { Item, ItemActions, ItemContent, ItemDescription } from "@/components/ui/item";
import { useSettingsContext } from "@/components/settings-provider";
import { api } from "@/lib/api";
import { CAPTIONS_LICENSE_NAME, CAPTIONS_NAME, CAPTIONS_URL } from "@/lib/captions";
import { SettingsCard } from "../settings-card";
import { ModelCard } from "./model-card";
import { ModelUsageRow } from "./model-usage-row";
import type { ModelStore } from "./stores";

export function CaptionsCard({ store }: { store: ModelStore }) {
  const { t } = useSettingsContext();
  return (
    <SettingsCard>
      <ModelCard title={CAPTIONS_NAME} description={t.captionsDescription} store={store} />
      <Item variant="muted" size="sm">
        <ItemContent>
          <ItemDescription>{t.cutoutLicenseLine.replace("{license}", CAPTIONS_LICENSE_NAME)}</ItemDescription>
        </ItemContent>
        <ItemActions>
          <Button variant="link" size="sm" onClick={() => api.openUrl(CAPTIONS_URL).catch(() => {})}>
            {t.cutoutLicenseRead}
          </Button>
        </ItemActions>
      </Item>
      <ModelUsageRow text={t.modelUsedCaptions} section="recording" label={`${t.recording} →`} />
    </SettingsCard>
  );
}
