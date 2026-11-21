import { invoke } from "@tauri-apps/api/core";

export type Settings = Record<string, any> & {
  ConfirmLaunches: boolean;
  ForceRobloxLanguage: boolean;
  Channel: string;
  ChannelChangeMode: number;
  RobloxDomain: string;
  StaticDirectory: boolean;
  CheckForUpdates: boolean;
  UseFastFlagManager: boolean;
  ApplyNonAllowlistedFlags: boolean;
  DebugDisableVersionPackageCleanup: boolean;
};

export type State = Record<string, any> & { ForceReinstall: boolean; LastPage: string | null };

export interface FlagEntry {
  Value: string;
  Enabled: boolean;
}
export interface FlagStore {
  Flags: Record<string, FlagEntry>;
}
export interface AllowlistEntry {
  name: string;
  category: string;
  note: string;
}
export interface Allowlist {
  source: string;
  retrieved: string;
  flags: AllowlistEntry[];
}

export interface GlobalSettings {
  framerateCap: number;
  graphicsQuality: number;
  transparency: number;
  reducedMotion: boolean;
  fontSize: number;
  mouseSensitivity: number;
  vrEnabled: boolean;
  readOnly: boolean;
}

export interface AppInfo {
  version: string;
  installed: boolean;
  runningFromInstall: boolean;
  fishstrapPresent: boolean;
  handlerIsOurs: boolean;
  importedFromFishstrap: boolean;
  baseDir: string;
}

export const api = {
  appInfo: () => invoke<AppInfo>("app_info"),
  install: () => invoke<void>("install_app"),
  importFishstrap: () => invoke<{ settings: boolean; flags: number; modFiles: number }>("import_fishstrap"),
  getSettings: () => invoke<Settings>("get_settings"),
  saveSettings: (value: Settings) => invoke<void>("save_settings", { value }),
  getState: () => invoke<State>("get_state"),
  saveState: (value: State) => invoke<void>("save_state", { value }),
  getFlags: () => invoke<{ store: FlagStore; allowlist: Allowlist }>("get_flags"),
  saveFlags: (store: FlagStore) => invoke<void>("save_flags", { store }),
  parseFlagImport: (text: string) => invoke<Record<string, string>>("parse_flag_import", { text }),
  flagCatalog: () => invoke<string[]>("flag_catalog"),
  refreshAllowlist: () => invoke<Allowlist>("refresh_allowlist"),
  getGlobalSettings: () => invoke<GlobalSettings>("get_global_settings"),
  saveGlobalSettings: (value: GlobalSettings) => invoke<void>("save_global_settings", { value }),
  channelInfo: (channel: string, domain: string) =>
    invoke<{ channel: string; version: string; guid: string }>("channel_info", { channel, domain }),
  launchRoblox: () => invoke<void>("launch_roblox"),
  openFolder: (which: "base" | "mods" | "logs" | "versions" | "roblox") => invoke<void>("open_folder", { which }),
  openUrl: (url: string) => invoke<void>("open_url", { url }),
};

export const FLAG_NAME_RE = /^(FFlag|DFFlag|SFFlag|FInt|DFInt|SFInt|FString|DFString|SFString|FLog|DFLog|SFLog)[A-Za-z0-9_]+$/;
