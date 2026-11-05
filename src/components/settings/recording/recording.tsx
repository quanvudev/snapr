import { useSettingsContext } from "@/components/settings-provider";
import { ModelStatusRow } from "../models/model-status-row";
import { useWhisperStore } from "../models/stores";
import { SectionHeading } from "../section-heading";
import { SettingsCard } from "../settings-card";
import { AutoCompressRow } from "./auto-compress-row";
import { AutoTranscribeRow } from "./auto-transcribe-row";
import { AutoTranslateRow } from "./auto-translate-row";
import { CountdownRow } from "./countdown-row";
import { FpsRow } from "./fps-row";
import { QualityRow } from "./quality-row";

export function Recording() {
  const { cfg, t } = useSettingsContext();
  const whisper = useWhisperStore();
  return (
    <>
      <SectionHeading>{t.whileRecording}</SectionHeading>
      <SettingsCard>
        <FpsRow />
        <QualityRow />
        <CountdownRow />
      </SettingsCard>
      <SectionHeading>{t.afterRecording}</SectionHeading>
      <SettingsCard>
        <AutoCompressRow />
        <AutoTranscribeRow />
        {cfg.record_transcribe && <AutoTranslateRow />}
        <ModelStatusRow store={whisper} />
      </SettingsCard>
    </>
  );
}
