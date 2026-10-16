import { Spinner } from "@/components/ui/spinner";
import icon from "../../../src-tauri/icons/128x128.png";

export function Splash({ version }: { version: string }) {
  return (
    <div data-tauri-drag-region className="flex h-screen flex-col items-center justify-center gap-2">
      <img src={icon} alt="" className="pointer-events-none size-20" />
      <div className="pointer-events-none text-xl font-semibold">snapr</div>
      <div className="pointer-events-none text-xs text-muted-foreground">{version}</div>
      <Spinner className="mt-3" />
    </div>
  );
}
