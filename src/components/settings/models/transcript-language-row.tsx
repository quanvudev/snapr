import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { useSettingsContext } from "@/components/settings-provider";
import type { TranscriptLanguage } from "@/lib/api";
import type { Strings } from "@/lib/queue";

const LANGUAGES: { value: TranscriptLanguage; label: keyof Strings }[] = [
  { value: "auto", label: "transcriptAuto" },
  { value: "vi", label: "transcriptVi" },
  { value: "en", label: "transcriptEn" },
];

export function TranscriptLanguageRow() {
  const { cfg, t, update } = useSettingsContext();
  const change = (value: string[]) => {
    const transcript_language = value[0] as TranscriptLanguage | undefined;
    if (transcript_language) update({ ...cfg, transcript_language });
  };

  return (
    <div className="flex basis-full items-center justify-between gap-3">
      <span className="text-sm">{t.transcriptLanguage}</span>
      <ToggleGroup variant="outline" size="sm" value={[cfg.transcript_language]} onValueChange={change}>
        {LANGUAGES.map((l) => (
          <ToggleGroupItem
            key={l.value}
            value={l.value}
            className="text-muted-foreground aria-pressed:border-primary aria-pressed:bg-primary aria-pressed:text-primary-foreground"
          >
            {t[l.label]}
          </ToggleGroupItem>
        ))}
      </ToggleGroup>
    </div>
  );
}
