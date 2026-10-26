import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { TooltipProvider } from "./components/ui/tooltip";
import { ToastProvider } from "./components/ui/toast";

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
      <App />
    </TooltipProvider>
    <ToastProvider />
  </React.StrictMode>,
);
