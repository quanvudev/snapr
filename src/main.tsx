import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { CameraApp } from "./components/camera";
import { PanelApp } from "./components/panel";
import { SplashApp } from "./components/splash";
import { TooltipProvider } from "./components/ui/tooltip";
import { Toaster } from "./components/ui/toast";

const VIEW = new URLSearchParams(window.location.search).get("view");
const BLOCKED_CTRL_KEYS = ["r", "p"];

if (import.meta.env.PROD) {
  document.addEventListener("contextmenu", (e) => e.preventDefault());
  document.addEventListener("keydown", (e) => {
    if (e.key === "F5" || (e.ctrlKey && BLOCKED_CTRL_KEYS.includes(e.key.toLowerCase()))) e.preventDefault();
  });
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <TooltipProvider>
      {VIEW === "panel" ? <PanelApp /> : VIEW === "camera" ? <CameraApp /> : VIEW === "splash" ? <SplashApp /> : <App />}
    </TooltipProvider>
    <Toaster />
  </React.StrictMode>,
);
