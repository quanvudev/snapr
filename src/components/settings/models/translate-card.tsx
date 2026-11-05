import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Item, ItemActions, ItemContent, ItemDescription } from "@/components/ui/item";
import { useSettingsContext } from "@/components/settings-provider";
import { api } from "@/lib/api";
import { HYMT_LICENSE_NAME, HYMT_LICENSE_URL, HYMT_NAME } from "@/lib/hymt";
import { SettingsCard } from "../settings-card";
import { LicenseDialog } from "./license-dialog";
import { ModelCard } from "./model-card";
import { ModelUsageRow } from "./model-usage-row";
import type { ModelStore } from "./stores";

type Props = { store: ModelStore; asking: boolean; onCloseLicense: () => void };

export function TranslateCard({ store, asking, onCloseLicense }: Props) {
  const { cfg, t } = useSettingsContext();
  const state = cfg.record_transcribe && cfg.record_translate ? t.modelOn : t.modelOff;
  return (
    <SettingsCard>
      <ModelCard title={HYMT_NAME} description={t.hymtDescription} store={store}>
        <LicenseDialog open={asking} bytes={store.bytes[store.selected] ?? 0} onClose={onCloseLicense} />
      </ModelCard>
      <Item variant="muted" size="sm">
        <ItemContent>
          <ItemDescription>{t.hymtLicenseLine.replace("{license}", HYMT_LICENSE_NAME)}</ItemDescription>
        </ItemContent>
        <ItemActions>
          <Button variant="link" size="sm" onClick={() => api.openUrl(HYMT_LICENSE_URL).catch(() => {})}>
            {t.hymtLicenseRead}
          </Button>
        </ItemActions>
      </Item>
      <ModelUsageRow text={t.modelUsedTranslate.replace("{state}", state)} section="recording" label={`${t.recording} →`} />
    </SettingsCard>
  );
}

export function useLicensePrompt() {
  const [asking, setAsking] = useState(false);
  return { asking, ask: () => setAsking(true), close: () => setAsking(false) };
}
