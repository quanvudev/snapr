import { useSettingsContext } from "@/components/settings-provider";
import { ItemGroup } from "@/components/ui/item";
import { BackgroundRow } from "./background-row";
import { PaddingRow } from "./padding-row";
import { RadiusRow } from "./radius-row";

export function Capture() {
  const { cfg } = useSettingsContext();
  return (
    <ItemGroup className="gap-3">
      <PaddingRow />
      {cfg.padding.enabled && (
        <>
          <RadiusRow />
          <BackgroundRow />
        </>
      )}
    </ItemGroup>
  );
}
