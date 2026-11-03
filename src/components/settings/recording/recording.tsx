import { ItemGroup } from "@/components/ui/item";
import { AutoCompressRow } from "./auto-compress-row";
import { AutoTranscribeRow } from "./auto-transcribe-row";
import { CountdownRow } from "./countdown-row";
import { FpsRow } from "./fps-row";
import { QualityRow } from "./quality-row";
import { TranscriptLanguageRow } from "./transcript-language-row";
import { WhisperModelRow } from "./whisper-model-row";

export function Recording() {
  return (
    <ItemGroup className="gap-3">
      <FpsRow />
      <QualityRow />
      <AutoCompressRow />
      <AutoTranscribeRow />
      <WhisperModelRow />
      <TranscriptLanguageRow />
      <CountdownRow />
    </ItemGroup>
  );
}
