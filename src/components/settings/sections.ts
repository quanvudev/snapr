import { Camera, Images, Info, Keyboard, Route, ScanText, Settings as SettingsIcon, Sparkles, Video } from "lucide-react";
import type { SectionId } from "@/hooks/use-settings";
import { About } from "./about/about";
import { Capture } from "./capture/capture";
import { Gallery } from "./gallery/gallery";
import { General } from "./general/general";
import { Hotkeys } from "./hotkeys/hotkeys";
import { Models } from "./models/models";
import { Recording } from "./recording/recording";
import { Ring } from "./ring/ring";
import { Text } from "./text/text";

export type SectionPlace = "top" | "settings" | "footer";

export const SECTIONS: {
  id: SectionId;
  icon: typeof Images;
  place: SectionPlace;
  component: () => React.ReactNode;
  wide?: boolean;
  fill?: boolean;
}[] = [
  { id: "gallery", icon: Images, place: "top", component: Gallery, wide: true, fill: true },
  { id: "general", icon: SettingsIcon, place: "settings", component: General },
  { id: "capture", icon: Camera, place: "settings", component: Capture },
  { id: "text", icon: ScanText, place: "settings", component: Text },
  { id: "recording", icon: Video, place: "settings", component: Recording },
  { id: "hotkeys", icon: Keyboard, place: "settings", component: Hotkeys },
  { id: "ring", icon: Route, place: "settings", component: Ring, wide: true },
  { id: "models", icon: Sparkles, place: "settings", component: Models },
  { id: "about", icon: Info, place: "footer", component: About },
];
