import { useEffect } from "react";
import { api, type Theme } from "@/lib/api";

const DARK_QUERY = "(prefers-color-scheme: dark)";

export function useTheme(theme: Theme = "system") {
  useEffect(() => {
    const query = window.matchMedia(DARK_QUERY);
    const apply = () => document.documentElement.classList.toggle("dark", theme === "dark" || (theme === "system" && query.matches));
    apply();
    api.setWindowTheme(theme);
    query.addEventListener("change", apply);
    return () => query.removeEventListener("change", apply);
  }, [theme]);
}
