# AGENTS.md

Guidance for AI coding agents (Claude Code, etc.) working in this repo. Read this before making changes.

## Project overview

**teddypicker** — a cross-platform desktop file manager (Windows/Linux/macOS) built with:
- **Backend**: Rust + Tauri v2
- **Frontend**: Svelte + TypeScript
- **Package manager**: Bun (not npm/yarn/pnpm)
- **Styling**: Tailwind CSS
- **UI components**: shadcn-svelte
- **Core feature**: an extension system supporting themes (CSS-only) and functional plugins (JS, e.g. file preview, custom context actions)

## Folder structure

```
teddypicker/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs
│   │   ├── fs_commands.rs        # IPC commands: read/write/move/delete/list files
│   │   ├── extension_manager.rs  # scans, validates, loads extension manifests
│   │   └── capabilities.rs       # per-extension permission checks before running a command
│   ├── tauri.conf.json
│   └── capabilities/             # Tauri v2 capability files (scoping fs/dialog/etc access)
├── src/                          # Svelte source
│   ├── lib/
│   │   ├── components/
│   │   │   └── ui/               # shadcn-svelte components (generated, don't hand-edit internals)
│   │   ├── FileGrid.svelte
│   │   ├── ExtensionHost.svelte  # sandboxed iframe host for JS plugins
│   │   └── ThemeProvider.svelte  # injects CSS variables from the active theme
│   ├── app.css                   # Tailwind entrypoint + theme tokens
│   └── App.svelte
├── components.json                # shadcn-svelte config
├── tailwind.config.ts
├── bun.lockb
└── extensions/                    # dev-time; runtime uses app_data_dir()/extensions
    ├── <theme-name>/
    │   ├── manifest.json
    │   └── theme.css
    └── <plugin-name>/
        ├── manifest.json
        └── plugin.js
```

## Key commands

```bash
bun install                  # install frontend dependencies
bun run tauri dev            # run the app in development mode
bun run tauri build          # build production binaries (per platform)
bun run check                # type-check Svelte (svelte-check)
bunx shadcn-svelte@latest add <component>   # add a new shadcn-svelte component
cargo check                  # check Rust without a full build (run from src-tauri/)
cargo clippy                 # lint Rust
cargo test                   # run Rust unit tests
```

Always run `cargo check` and `bun run check` after touching Rust or Svelte code, before considering a task done. Never fall back to `npm`/`npx`/`yarn`/`pnpm` in this repo — Bun is the only package manager and script runner.

## Code conventions

- **Rust**: follow default `rustfmt` style (`cargo fmt` before committing). IPC commands (`#[tauri::command]`) live in domain-specific files (`fs_commands.rs`, etc.), not all in `main.rs`.
- **Svelte**: components use PascalCase (`FileGrid.svelte`). Cross-component state uses Svelte stores, not deep prop-drilling.
- **Styling**: use Tailwind utility classes directly in markup; avoid writing new custom CSS unless Tailwind genuinely can't express it. Design tokens (colors, radius, etc.) live in `app.css` / `tailwind.config.ts` as CSS variables so themes can override them.
- **UI components**: prefer an existing shadcn-svelte component (`src/lib/components/ui/`) over building one from scratch. If a needed component isn't installed yet, add it with `bunx shadcn-svelte@latest add <component>` rather than hand-rolling an equivalent.
- **IPC command naming**: `snake_case` in Rust, called from JS with the exact same string via `invoke("command_name", {...})`.
- Don't expose a new Rust command without registering its capability in `src-tauri/capabilities/`.

## Extension system

There are two extension types, distinguished by the `"type"` field in `manifest.json`:

### 1. Theme (`"type": "theme"`)
- Contains only `manifest.json` + a CSS file.
- The CSS overrides design tokens (`--surface-0`, `--surface-1`, `--text-primary`, etc.) that the whole UI — including Tailwind's theme layer — consumes.
- Runs no JS at all → no sandbox needed, safe by default.
- Loaded by `ThemeProvider.svelte`, which injects a `<link>`/`<style>` at the root.

### 2. Functional plugin (`"type": "plugin"`)
- Contains `manifest.json` + a JS entry file (`plugin.js`).
- Can register: preview handlers for specific file types, custom context-menu actions.
- **Must run in a sandbox** (`<iframe sandbox="allow-scripts">` via `ExtensionHost.svelte`), never directly in the main DOM.
- Plugin ↔ app communication only happens via `postMessage` through a restricted API defined in `ExtensionHost.svelte` (e.g. `previewFile(path)`, `getSelectedFiles()`, `showToast(msg)`) — never direct access to `window.__TAURI__`.
- The parent-side bridge translates those messages into Tauri `invoke()` calls, and **permissions declared in the manifest (`fs:read`, etc.) are checked twice**: once in the JS bridge, once again in the Rust command via `capabilities.rs`. Don't skip either layer.

### Manifest format

```json
{
  "id": "unique-extension-id",
  "name": "Display Name",
  "version": "1.0.0",
  "type": "theme | plugin",
  "entry": "theme.css | plugin.js",
  "permissions": ["fs:read"],
  "contributes": {
    "previewHandlers": ["png", "jpg"],
    "contextMenuActions": [{ "label": "...", "command": "..." }]
  }
}
```

`extension_manager.rs` validates this manifest at startup and whenever the extensions folder changes (if a watcher is implemented).

## Security rules — do not violate

1. Plugin JS must **never** get a direct reference to `window.__TAURI__` or any native filesystem API.
2. Every new Rust command must be checked against the capability the extension actually declared in its manifest — never trust it blindly.
3. Commands touching the filesystem must be scope-limited (e.g. can't read outside the directory the user currently has open) via Tauri v2 capability/scope config, not manual validation that's easy to bypass.
4. When adding new extension API surface, add a new granular permission to the manifest schema — don't reuse an overly broad permission (e.g. `"fs:*"`).

## Testing

- New Rust commands → add a `cargo test` unit test, especially for manifest validation and permission checks.
- Svelte UI changes touching extension loading → manually test with at least one sample theme and one sample plugin in the `extensions/` folder.

## Confirm with the user before changing

- The extension manifest schema (adding/changing fields) — this is a breaking change for existing extensions.
- The list of permissions available to plugins.
- The Tauri v2 capability file structure under `src-tauri/capabilities/`.
