# teddypicker

A fast, cross-platform desktop file manager built with **Tauri v2** and **Svelte**, with a built-in extension system for themes and functional plugins.

## Features

- 🗂️ Native file browsing — list, create, rename, move, copy, delete
- 🗑️ Trash/recycle bin integration (list, restore, purge)
- 🖼️ Lazy, virtualized thumbnail loading — safe on folders with thousands of files
- 🎨 Custom titlebar with native window controls (minimize / maximize / close)
- 🧩 Extension system — install themes or functional plugins without rebuilding the app
- 💻 Cross-platform — Windows, Linux, macOS

## Tech stack

| Layer | Choice |
|---|---|
| Backend | Rust + Tauri v2 |
| Frontend | Svelte 5 (runes) + TypeScript |
| Styling | Tailwind CSS |
| UI components | shadcn-svelte |
| Package manager | Bun |

## Getting started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable toolchain)
- [Bun](https://bun.sh)
- Platform build dependencies for Tauri — see the [Tauri prerequisites guide](https://tauri.app/start/prerequisites/)

### Setup

```bash
bun install
bun run tauri dev
```

### Build for production

```bash
bun run tauri build
```

## Project structure

```
teddypicker/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs / lib.rs
│   │   ├── fs_commands.rs        # read/write/move/delete/list file commands
│   │   ├── extension_manager.rs  # scans, validates, loads extension manifests
│   │   └── capabilities.rs       # per-extension permission checks
│   ├── tauri.conf.json
│   └── capabilities/             # Tauri v2 capability files
├── src/
│   ├── lib/
│   │   ├── components/
│   │   │   ├── ui/                # shadcn-svelte components
│   │   │   ├── Titlebar.svelte
│   │   │   └── AppLayout.svelte
│   │   └── ExtensionHost.svelte   # sandboxed iframe host for plugins
│   └── App.svelte
└── extensions/                    # dev-time extensions (themes & plugins)
```

See [`AGENTS.md`](./AGENTS.md) for detailed architecture notes, coding conventions, and the extension manifest format.

## Extensions

teddypicker supports two kinds of extensions, dropped into the `extensions/` folder (or the app's data directory at runtime):

- **Theme** — CSS-only, overrides design tokens (`--surface-0`, `--text-primary`, etc.)
- **Plugin** — JS-based, runs sandboxed in an iframe, can register file previews and custom context-menu actions

Full manifest format and security model are documented in [`AGENTS.md`](./AGENTS.md#sistem-extension).

## Contributing

Issues and pull requests are welcome. Please run `cargo check`, `cargo clippy`, and `bun run check` before submitting a PR.

## License

_TBD_
