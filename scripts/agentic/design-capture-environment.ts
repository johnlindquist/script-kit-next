import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

/**
 * Create a disposable Script Kit home whose window pixels do not depend on
 * the desktop behind the app or on an animated background shader.
 */
export function createOpaqueDesignCaptureHome(prefix: string): string {
  const home = mkdtempSync(join(tmpdir(), prefix));
  const kitDir = join(home, ".scriptkit");
  mkdirSync(kitDir, { recursive: true });
  writeFileSync(
    join(kitDir, "config.ts"),
    'export default { effects: { background: "off", intensity: 0 } };\n',
  );
  writeFileSync(
    join(kitDir, "theme.json"),
    `${JSON.stringify({
      appearance: "dark",
      colors: {
        background: {
          main: "#0F0F0F",
          title_bar: "#1A1A1A",
          search_box: "#2A2A2A",
          log_panel: "#080808",
        },
        text: {
          primary: "#FFFFFF",
          secondary: "#FFFFFF",
          tertiary: "#FFFFFF",
          muted: "#FFFFFF",
          dimmed: "#FFFFFF",
          on_accent: "#0F0F0F",
        },
        accent: { selected: "#FBBF24", selected_subtle: "#5A5A5A" },
        ui: {
          border: "#343434",
          success: "#00FF00",
          error: "#EF4444",
          warning: "#F59E0B",
          info: "#3B82F6",
        },
      },
      vibrancy: {
        // Preserve the production borderless main-window geometry. A fully
        // opaque vibrancy_background below prevents desktop pixels from
        // contributing while WindowBackgroundAppearance stays Blurred.
        enabled: true,
        material: "menu",
        backdrop_saturation: 2.6,
      },
      opacity: {
        main: 0.5,
        title_bar: 0.5,
        search_box: 0.5,
        log_panel: 0.5,
        selected: 0.2,
        hover: 0.06,
        preview: 0.5,
        dialog: 0.5,
        input: 0.5,
        panel: 0.5,
        input_inactive: 0.5,
        input_active: 0.5,
        border_inactive: 0.125,
        border_active: 0.25,
        vibrancy_background: 1.0,
        text_name: 1.0,
        text_strong: 0.8,
        text_muted_alpha: 0.65,
        text_hint: 0.45,
        text_placeholder: 0.4,
        text_icon: 0.5,
      },
      background_gradient: null,
    }, null, 2)}\n`,
  );
  const windowStateDir = join(home, ".sk", "kit");
  mkdirSync(windowStateDir, { recursive: true });
  writeFileSync(
    join(windowStateDir, "window-state.json"),
    `${JSON.stringify({
      version: 3,
      main: {
        mode: "windowed",
        x: 100,
        y: 100,
        width: 750,
        height: 480,
      },
      main_per_display: {},
      notes: null,
      notes_per_display: {},
      ai: null,
      ai_per_display: {},
      ai_mini: null,
      agent_chat: null,
    }, null, 2)}\n`,
  );
  return home;
}

export function opaqueDesignCaptureEnv(home: string): Record<string, string> {
  return {
    HOME: home,
    SK_PATH: join(home, ".scriptkit"),
    SCRIPT_KIT_DEBUG_NO_GLASS: "1",
  };
}
