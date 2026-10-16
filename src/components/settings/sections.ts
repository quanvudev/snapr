import { Eye, Frame, Info, Keyboard, Settings as SettingsIcon, Video } from "lucide-react";
import type { SectionId } from "@/hooks/use-settings";
import { About } from "./about/about";
import { Capture } from "./capture/capture";
import { General } from "./general/general";
import { Hotkeys } from "./hotkeys/hotkeys";
import { Preview } from "./preview/preview";
import { Recording } from "./recording/recording";

export const SECTIONS: { id: SectionId; icon: typeof Eye; component: () => React.ReactNode }[] = [
  { id: "general", icon: SettingsIcon, component: General },
  { id: "capture", icon: Frame, component: Capture },
  { id: "preview", icon: Eye, component: Preview },
  { id: "recording", icon: Video, component: Recording },
  { id: "hotkeys", icon: Keyboard, component: Hotkeys },
  { id: "about", icon: Info, component: About },
];
