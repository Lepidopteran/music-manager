# Muusik

## Stack

- **Backend**: Rust (edition 2024), Axum, SQLite (sqlx offline), Handlebars templates
- **Frontend**: Svelte 5, TypeScript, Tailwind CSS 4, Vite, Vitest
- **JS pkg manager**: pnpm only (`build.rs` calls `pnpm`, not npm)
- **Formatter**: `dprint` (tabs, `dprint.json` config) for TS/MD/TOML/Markup; Prettier for `.svelte`
- **Linter**: oxlint (frontend, config at `frontend/.oxlintrc.json`)
- **Type bindings**: auto-generated via `ts-rs` — **never edit** `frontend/src/lib/bindings/*.ts` by hand

## Commands

| Where | Command | What |
|---|---|---|
| root | `cargo run` | Build + run backend. `build.rs` runs `pnpm build` first — frontend embedded via `rust-embed` into binary |
| root | `cargo test` | Rust tests (live inside `src/`, not `tests/`) |
| root | `dprint check` | Format-check all |
| root | `dprint fmt` | Auto-format |
| `frontend/` | `pnpm dev` | Vite dev on `:5173`, proxies `/api` → `:3000` |
| `frontend/` | `pnpm check` | `svelte-check` + `tsc` |
| `frontend/` | `pnpm test` | Vitest |
| `frontend/` | `npx oxlint` | Frontend lint |

Run both `cargo run` (backend) and `pnpm dev` (frontend) separately for development; Vite handles the proxy.

## Key directories

| Path | Role |
|---|---|
| `src/` | Rust backend: Axum routes, DB, metadata, jobs, organize |
| `frontend/src/` | Svelte 5 frontend |
| `frontend/src/lib/bindings/` | Auto-generated TS types from Rust `#[derive(TS)]` — do not edit |
| `migrations/` | sqlx SQLite migrations |
| `templates/` | Handlebars config templates |
| `frontend/plugins/icons.ts` | Custom Vite plugin for bundling icons |

## Database

- SQLite only. Offline mode on (`SQLX_OFFLINE=true` in `.cargo/config.toml`).
- No live DB needed at compile time; `.sqlx/` caches queries.
- New migration: `sqlx migrate add -r <name>` (needs sqlx-cli + live DB).
- Some migrations run custom Rust after-logic (defined in `src/migration.rs` via `CUSTOM_MIGRATIONS`).

## Architecture quirks

- `build.rs` runs `pnpm build` — `cargo build` implicitly builds frontend → `dist/` (embedded in binary via `rust-embed`).
- Config: `config.toml` (TOML). Override paths via `MUUSIK_CONFIG_DIR`, `MUUSIK_DATA_DIR`, `MUUSIK_CACHE_DIR` env vars.
- Config template uses Handlebars `{{ }}` syntax.
- SSE events at `/api/events` for real-time job + file-operation updates.
- Svelte 5: uses `mount()` API (not `new App()`), runes syntax expected.

## Conventions

- Commits: Conventional Commits.
- Rust edition 2024 (not 2021).
- Path aliases in frontend TS: `@api/*`, `@components/*`, `@lib/*`, `@pages/*`, `@utils/*`, `@assets/*`, `@attachments/*`, `@state`.
- `pnpm-workspace.yaml` allows builds for `dprint` and `esbuild` only.
