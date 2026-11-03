import { useSettingsContext } from "@/components/settings-provider";
import { ClipboardRow } from "./clipboard-row";
import { FolderRow } from "./folder-row";
import { LanguageRow } from "./language-row";
import { PreviewRow } from "./preview-row";
import { StartupRow } from "./startup-row";
import { ThemeRow } from "./theme-row";
import { UpdateCheckRow } from "./update-check-row";
import { SectionHeading } from "../section-heading";
import { SettingsCard } from "../settings-card";

export function General() {
  const { t } = useSettingsContext();
  return (
    <>
      <SectionHeading>{t.output}</SectionHeading>
      <SettingsCard>
        <FolderRow withOpen />
        <ClipboardRow />
        <PreviewRow />
      </SettingsCard>
      <SectionHeading>{t.system}</SectionHeading>
      <SettingsCard>
        <StartupRow />
        <LanguageRow />
        <ThemeRow />
        <UpdateCheckRow />
      </SettingsCard>
    </>
  );
}
