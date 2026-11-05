import { useSettingsContext } from "@/components/settings-provider";
import { SectionHeading } from "../section-heading";
import { SettingsCard } from "../settings-card";
import { BackgroundRow } from "./background-row";
import { PaddingRow } from "./padding-row";
import { RadiusRow } from "./radius-row";
import { RedactionCard } from "./redaction-card";

export function Capture() {
  const { cfg, t } = useSettingsContext();
  return (
    <>
      <SettingsCard>
        <PaddingRow />
        {cfg.padding.enabled && <RadiusRow />}
        {cfg.padding.enabled && <BackgroundRow />}
      </SettingsCard>
      <SectionHeading>{t.redaction}</SectionHeading>
      <RedactionCard />
    </>
  );
}
