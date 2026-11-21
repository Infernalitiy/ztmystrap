# ztmystrap

A fast, open-source Roblox bootstrapper for Windows, written in Rust (Tauri 2 + Svelte 5).

ztmystrap installs and updates Roblox itself, applies your FastFlags and mods, and launches the game. It is a
from-scratch Rust reimplementation of [Fishstrap](https://github.com/fishstrap/fishstrap), which is a fork of
[Bloxstrap](https://github.com/bloxstraplabs/bloxstrap). The goal is feature parity first, then performance features.

> ztmystrap is not affiliated with Roblox Corporation. It never injects into or reads the memory of the Roblox process.

## Features (v0.1)

- Installs and updates Roblox from the official CDN: parallel, MD5-verified downloads, reuse of cached packages, and automatic CDN mirror selection.
- Handles `roblox:` / `roblox-player:` links. When uninstalled, it hands them back to Fishstrap or the stock launcher.
- Channel switching (automatic, prompt or ignore), a custom Roblox domain, a static version folder and force reinstall.
- **FastFlag editor** with two sections:
  - **Allowlisted:** flags Roblox actually applies ([official list](https://devforum.roblox.com/t/allowlist-for-local-client-configuration-via-fast-flags/3966569)), refreshable from this repo's `remote-config.json`.
  - **Non-allowlisted:** every other flag. You can search all ~18k Roblox flags, toggle each one, and they are written only if you opt in.
  - Presets: rendering API, MSAA, texture quality, DPI fix, quality override, mesh LOD, grass and gray sky.
  - JSON import (tolerant of comments and trailing commas) and export to the clipboard.
- **Global Settings** editor for Roblox's in-game settings file: FPS cap, graphics level, sensitivity, transparency, text size, reduced motion, VR, and a read-only lock.
- Mods folder mirrored into Roblox. Removed mods are restored from the original package.
- One-click import of your Fishstrap settings, FastFlags and mods.

## Roadmap

| Version | Scope |
|---|---|
| v0.2 | Mod presets (fonts, emoji, cursors, sounds), cache cleaner, shortcuts, appearance and bootstrapper styles |
| v0.3 | Tray watcher, activity tracking, Discord Rich Presence and invites, server location, better matchmaking, window tweaks |
| v0.4 | Roblox Studio, VNG, background updater, self-update, uninstaller polish |
| Later | Performance: process priority and affinity, timer resolution, power plans, GPU preference, FPS monitor, plus account switcher, profiles and CLI |

## Install

Download `ztmystrap.exe` from [Releases](https://github.com/Infernalitiy/ztmystrap/releases), run it and click **Install**.
It installs to `%LOCALAPPDATA%\ztmystrap`. Builds are unsigned for now, so Windows SmartScreen may warn you.

## Build from source

Requirements: Rust (stable, MSVC), Node.js 20+, and the Visual Studio C++ Build Tools.

```powershell
npm install
npx tauri dev                 # run in development
npx tauri build --no-bundle   # src-tauri\target\release\ztmystrap.exe
cd src-tauri; cargo test      # unit tests
```

Set `ZTMYSTRAP_HOME` to a scratch folder to test the bootstrapper without touching your real install.

## Credits

- [Bloxstrap](https://github.com/bloxstraplabs/bloxstrap) by pizzaboxer and contributors (MIT)
- [Fishstrap](https://github.com/fishstrap/fishstrap) by returnrqt and contributors (MIT)
- FastFlag catalog from [MaximumADHD/Roblox-Client-Tracker](https://github.com/MaximumADHD/Roblox-Client-Tracker)

## License

MIT. See [LICENSE](LICENSE).
