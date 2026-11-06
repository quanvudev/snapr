import { useEffect, useState } from "react";
import { Spinner } from "@/components/ui/spinner";
import { api } from "@/lib/api";
import { cn } from "@/lib/utils";
import "@/App.css";
import icon from "../../../src-tauri/icons/128x128@2x.png";

export function SplashApp() {
  const [done, setDone] = useState(false);

  useEffect(() => {
    for (const el of [document.documentElement, document.body, document.getElementById("root")]) {
      if (!el) continue;
      el.style.background = "transparent";
      el.style.overflow = "hidden";
    }
    const un = api.onSplashDone(() => setDone(true));
    return () => {
      un.then((f) => f());
    };
  }, []);

  return (
    <div
      className={cn(
        "pointer-events-none flex h-screen flex-col items-center justify-center gap-5 select-none",
        done ? "animate-out fade-out zoom-out-95 fill-mode-forwards duration-200" : "animate-in fade-in zoom-in-90 duration-500",
      )}
    >
      <img src={icon} alt="" className="size-24 drop-shadow-[0_8px_24px_rgba(0,0,0,0.35)]" />
      <Spinner className="size-5 text-white drop-shadow-[0_1px_2px_rgba(0,0,0,0.6)]" />
    </div>
  );
}
