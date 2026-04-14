import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

let devPanelWindow: WebviewWindow | null = null;

export function useDevPanel() {
  async function openDevPanel() {
    // Check if window already exists and is valid
    if (devPanelWindow) {
      try {
        await devPanelWindow.setFocus();
        return;
      } catch {
        // Window was closed, recreate it
        devPanelWindow = null;
      }
    }

    devPanelWindow = new WebviewWindow("dev-panel", {
      url: "/dev-panel.html",
      title: "glogger - Dev Panel",
      width: 800,
      height: 600,
      center: true,
    });

    devPanelWindow.once("tauri://error", () => {
      devPanelWindow = null;
    });

    devPanelWindow.once("tauri://destroyed", () => {
      devPanelWindow = null;
    });
  }

  return { openDevPanel };
}
