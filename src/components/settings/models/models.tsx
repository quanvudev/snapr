import { useSettingsContext } from "@/components/settings-provider";
import { SectionHeading } from "../section-heading";
import { AccelerationCard } from "./acceleration-card";
import { SettingsCard } from "../settings-card";
import { CutoutCard } from "./cutout-card";
import { DownloadedModels } from "./downloaded-models";
import { ModelCard } from "./model-card";
import { ModelUsageRow } from "./model-usage-row";
import { paddleLabels } from "./paddle-labels";
import { useCutoutStore, useRuntimeStore, usePaddleStore, useTranslateStore, useWhisperGpuStore, useWhisperStore } from "./stores";
import { TranslateCard, useLicensePrompt } from "./translate-card";
import { TranscriptLanguageRow } from "./transcript-language-row";

export function Models() {
  const { cfg, t } = useSettingsContext();
  const paddle = usePaddleStore();
  const whisper = useWhisperStore();
  const license = useLicensePrompt();
  const translate = useTranslateStore(license.ask);
  const cutout = useCutoutStore();
  const whisperGpu = useWhisperGpuStore();
  const runtimes = useRuntimeStore();
  const { descs } = paddleLabels(t);
  const paddleDescription = `${descs[paddle.selected as keyof typeof descs]} ${t.paddleNoVi}`;
  const state = (on: boolean) => (on ? t.modelOn : t.modelOff);

  return (
    <>
      <p className="text-sm text-muted-foreground">{t.modelsIntro}</p>
      <SectionHeading>{t.accelTitle}</SectionHeading>
      <p className="text-sm text-muted-foreground">{t.accelHint}</p>
      <AccelerationCard />
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
      <SectionHeading>{t.modelsTranslation}</SectionHeading>
      <TranslateCard store={translate} asking={license.asking} onCloseLicense={license.close} />
      <SectionHeading>{t.modelsCutout}</SectionHeading>
      <CutoutCard store={cutout} />
      <DownloadedModels stores={[paddle, whisper, whisperGpu, translate, cutout, runtimes]} />
    </>
  );
}
