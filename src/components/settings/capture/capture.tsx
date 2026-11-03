import { useSettingsContext } from "@/components/settings-provider";
import { SettingsCard } from "../settings-card";
import { BackgroundRow } from "./background-row";
import { PaddingRow } from "./padding-row";
import { RadiusRow } from "./radius-row";

export function Capture() {
  const { cfg } = useSettingsContext();
  return (
    <SettingsCard>
      <PaddingRow />
      {cfg.padding.enabled && <RadiusRow />}
      {cfg.padding.enabled && <BackgroundRow />}
    </SettingsCard>
  );
}
