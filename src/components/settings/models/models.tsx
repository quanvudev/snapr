import { useSettingsContext } from "@/components/settings-provider";
import { SectionHeading } from "../section-heading";
import { SettingsCard } from "../settings-card";
import { DownloadedModels } from "./downloaded-models";
import { ModelCard } from "./model-card";
import { ModelUsageRow } from "./model-usage-row";
import { paddleLabels } from "./paddle-labels";
import { usePaddleStore, useWhisperStore } from "./stores";
import { TranscriptLanguageRow } from "./transcript-language-row";

export function Models() {
  const { cfg, t } = useSettingsContext();
  const paddle = usePaddleStore();
  const whisper = useWhisperStore();
  const { descs } = paddleLabels(t);
  const paddleDescription = `${descs[paddle.selected as keyof typeof descs]} ${t.paddleNoVi}`;
  const state = (on: boolean) => (on ? t.modelOn : t.modelOff);

  return (
    <>
      <p className="text-sm text-muted-foreground">{t.modelsIntro}</p>
      <SectionHeading>{t.modelsText}</SectionHeading>
      <SettingsCard>
        <ModelCard title={t.paddleModel} description={paddleDescription} store={paddle} />
        <ModelUsageRow
          text={t.modelUsedText.replace("{state}", state(cfg.text_capture && cfg.ocr_engine === "paddle"))}
          section="text"
          label={`${t.text} →`}
        />
      </SettingsCard>
      <SectionHeading>{t.modelsSpeech}</SectionHeading>
      <SettingsCard>
        <ModelCard title={t.whisperModel} store={whisper}>
          <TranscriptLanguageRow />
        </ModelCard>
        <ModelUsageRow
          text={t.modelUsedSpeech.replace("{state}", state(cfg.record_transcribe))}
          section="recording"
          label={`${t.recording} →`}
        />
      </SettingsCard>
      <DownloadedModels stores={[paddle, whisper]} />
    </>
  );
}
