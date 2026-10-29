import { useEffect, useRef } from "react";
import { api } from "@/lib/api";
import "@/App.css";

export function CameraApp() {
  const video = useRef<HTMLVideoElement>(null);

  useEffect(() => {
    for (const el of [document.documentElement, document.body, document.getElementById("root")]) {
      if (!el) continue;
      el.style.background = "transparent";
      el.style.overflow = "hidden";
    }
    let stream: MediaStream | null = null;
    let current: string | null = null;
    let alive = true;

    const open = async (device: string) => {
      if (device === current) return;
      current = device;
      stream?.getTracks().forEach((track) => track.stop());
      stream = null;
      if (video.current) video.current.srcObject = null;
      if (!device) return;
      try {
        const next = await navigator.mediaDevices.getUserMedia({ video: { deviceId: { exact: device } } });
        if (!alive || current !== device) {
          next.getTracks().forEach((track) => track.stop());
          return;
        }
        stream = next;
        if (video.current) video.current.srcObject = next;
      } catch {
        current = null;
      }
    };

    open(new URLSearchParams(window.location.search).get("device") ?? "");
    const unlisten = api.onRecordPanel((s) => open(s.camera));
    return () => {
      alive = false;
      unlisten.then((f) => f());
      stream?.getTracks().forEach((track) => track.stop());
    };
  }, []);

  return (
    <div data-tauri-drag-region className="h-screen w-screen p-7">
      <div data-tauri-drag-region className="size-full rounded-full drop-shadow-[0_10px_12px_rgba(0,0,0,0.6)]">
        <div
          data-tauri-drag-region
          className="size-full overflow-hidden rounded-full"
          style={{ maskImage: "radial-gradient(circle closest-side, #000 calc(100% - 2px), transparent 100%)" }}
        >
          <video
            ref={video}
            autoPlay
            muted
            playsInline
            className="pointer-events-none size-full scale-y-[1.04] -scale-x-[1.04] object-cover"
          />
        </div>
      </div>
    </div>
  );
}
