import { api, type Allowlist, type AppInfo, type FlagStore, type GlobalSettings, type Settings, type State } from "./api";

/** Everything the settings window edits. Saved together with the Save button. */
export const app = $state({
  loaded: false,
  info: null as AppInfo | null,
  settings: null as Settings | null,
  state: null as State | null,
  flags: { Flags: {} } as FlagStore,
  allowlist: { source: "", retrieved: "", flags: [] } as Allowlist,
  gbs: null as GlobalSettings | null,
  gbsError: "",
  dirty: false,
  toast: "",
});

export async function loadAll() {
  const [info, settings, state, flags] = await Promise.all([api.appInfo(), api.getSettings(), api.getState(), api.getFlags()]);
  app.info = info;
  app.settings = settings;
  app.state = state;
  app.flags = flags.store;
  app.allowlist = flags.allowlist;
  try {
    app.gbs = await api.getGlobalSettings();
  } catch (e) {
    app.gbsError = String(e);
  }
  app.loaded = true;
  app.dirty = false;
}

export function markDirty() {
  app.dirty = true;
}

export function toast(msg: string) {
  app.toast = msg;
  setTimeout(() => {
    if (app.toast === msg) app.toast = "";
  }, 3500);
}

export async function saveAll() {
  if (!app.settings || !app.state) return;
  await api.saveSettings(app.settings);
  await api.saveState(app.state);
  await api.saveFlags(app.flags);
  if (app.gbs) await api.saveGlobalSettings(app.gbs);
  app.dirty = false;
  toast("Settings saved");
}

export function allowlisted(name: string) {
  return app.allowlist.flags.some((f) => f.name === name);
}

/** Set a flag (enabled) or remove it with `null`. */
export function setFlag(name: string, value: string | null) {
  if (value === null) delete app.flags.Flags[name];
  else app.flags.Flags[name] = { Value: value, Enabled: true };
  markDirty();
}

export function flagValue(name: string): string | null {
  return app.flags.Flags[name]?.Value ?? null;
}
