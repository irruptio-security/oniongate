import type { MouseEvent } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";

// WKWebView ignores `-webkit-app-region`, and `data-tauri-drag-region` is
// unreliable with `titleBarStyle: Overlay` (it drops mousedown depending on
// z-order). Starting the native drag explicitly on mousedown is the reliable
// approach on macOS. Interactive elements opt out via `data-no-drag` or by
// being a standard control.
export function startWindowDrag(event: MouseEvent) {
  if (event.button !== 0) return;
  const target = event.target as HTMLElement | null;
  if (
    target?.closest(
      "button, a, input, select, textarea, label, [role='button'], [data-no-drag]",
    )
  ) {
    return;
  }
  void getCurrentWindow()
    .startDragging()
    .catch(() => {
      /* window may be unavailable outside Tauri (e.g. plain browser dev) */
    });
}
