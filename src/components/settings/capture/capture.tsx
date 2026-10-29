import { ItemGroup } from "@/components/ui/item";
import { BackgroundRow } from "./background-row";
import { PaddingRow } from "./padding-row";
import { RadiusRow } from "./radius-row";
import { SizeRow } from "./size-row";
import { TextCaptureRow } from "./text-capture-row";
import { WallpaperRow } from "./wallpaper-row";

export function Capture() {
  return (
    <ItemGroup className="gap-3">
      <TextCaptureRow />
      <PaddingRow />
      <SizeRow />
      <RadiusRow />
      <BackgroundRow />
      <WallpaperRow />
    </ItemGroup>
  );
}
