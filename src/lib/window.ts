import {
  currentMonitor,
  getCurrentWindow,
  LogicalSize,
} from "@tauri-apps/api/window";

/** Sidebar rail widths (must match the Tailwind widths in App.tsx). */
export const SIDEBAR_EXPANDED_PX = 224; // w-56
export const SIDEBAR_COLLAPSED_PX = 88; // w-[88px] — wide enough for the macOS traffic lights
const SIDEBAR_DELTA = SIDEBAR_EXPANDED_PX - SIDEBAR_COLLAPSED_PX;

export const SIDEBAR_STORAGE_KEY = "oniongate.sidebar.collapsed";

const clamp = (value: number, min: number, max: number) =>
  Math.min(Math.max(value, min), max);

// The window width when the sidebar is expanded, chosen at launch from the
// device screen. Collapsing subtracts the rail delta so the content area keeps
// the same width while the whole window narrows (ExpressVPN-style).
let baseWidth = 1040;
let baseHeight = 700;

export function readSidebarCollapsed(): boolean {
  try {
    return localStorage.getItem(SIDEBAR_STORAGE_KEY) === "1";
  } catch {
    return false;
  }
}

/**
 * Size the window to a fixed preset derived from the device's screen, apply the
 * current collapsed state, then center and reveal it. The window is created with
 * `visible: false` and is not user-resizable, so this runs once at startup.
 */
export async function applyPresetWindowSize(collapsed: boolean): Promise<void> {
  const win = getCurrentWindow();
  try {
    const monitor = await currentMonitor();
    if (monitor) {
      const scale = monitor.scaleFactor || 1;
      const screenW = monitor.size.width / scale;
      const screenH = monitor.size.height / scale;

      // ~3:2 preset, scaled to the screen and clamped to sane bounds.
      baseWidth = clamp(Math.round(screenW * 0.62), 900, 1200);
      baseHeight = clamp(Math.round(screenH * 0.72), 620, 820);

      // Never exceed the visible screen (leave room for menu bar / dock).
      baseWidth = Math.min(baseWidth, Math.round(screenW - 40));
      baseHeight = Math.min(baseHeight, Math.round(screenH - 80));
    }

    const width = collapsed ? baseWidth - SIDEBAR_DELTA : baseWidth;
    await win.setSize(new LogicalSize(width, baseHeight));
    await win.center();
  } catch (error) {
    console.error("Failed to apply preset window size", error);
  } finally {
    try {
      await win.show();
    } catch {
      /* window may already be visible */
    }
  }
}

/**
 * Grow/shrink the window width by the sidebar delta when the rail is toggled,
 * keeping the top-left corner fixed (no re-center) so it feels like the rail is
 * folding into/out of the window edge.
 */
export async function setWindowCollapsed(collapsed: boolean): Promise<void> {
  const win = getCurrentWindow();
  const width = collapsed ? baseWidth - SIDEBAR_DELTA : baseWidth;
  try {
    await win.setSize(new LogicalSize(width, baseHeight));
  } catch (error) {
    console.error("Failed to resize window for sidebar toggle", error);
  }
}
