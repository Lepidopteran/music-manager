# Muusik

> **Quick Start for Agents**: This project uses Rust backend + Svelte 5 frontend. Run `cargo run` for full build+run. Frontend types auto-sync with Rust via `ts-rs` — **never edit** `frontend/src/lib/bindings/*.ts`.

---

## Project Overview

**Muusik** is a music manager built with:

| Layer           | Technology                                                             |
| --------------- | ---------------------------------------------------------------------- |
| **Backend**     | Rust (edition 2024), Axum, SQLite (sqlx offline), Handlebars templates |
| **Frontend**    | Svelte 5, TypeScript, Tailwind CSS 4, Vite, Vitest                     |
| **Package Mgr** | pnpm only (`build.rs` calls `pnpm`, not npm)                           |
| **Formatter**   | `dprint` (tabs) + Prettier (`.svelte` files only)                      |
| **Linter**      | oxlint (frontend, config at `frontend/.oxlintrc.json`)                 |

---

## Commands Reference

| Command        | Description                                                                                              |
| -------------- | -------------------------------------------------------------------------------------------------------- |
| `cargo run`    | Build + run backend. `build.rs` runs `pnpm build` first — frontend embedded via `rust-embed` into binary |
| `cargo build`  | Build backend + frontend (embeds into binary)                                                            |
| `cargo test`   | Run Rust tests (live inside `src/`, not `tests/`)                                                        |
| `dprint check` | **Format-check all files** (respects `dprint.json` excludes)                                             |
| `dprint fmt`   | **Auto-format all files** (respects `dprint.json` excludes)                                              |
| `cargo check`  | Fast compilation check (no binary output)                                                                |
| `cargo clippy` | Run Clippy lints (Rust)                                                                                  |

> **Frontend commands**: See [`frontend/AGENTS.md`](frontend/AGENTS.md) for Vite dev server, testing, and Svelte-specific commands.

---

## Directory Structure

| Path                             | Role                                                                          |
| -------------------------------- | ----------------------------------------------------------------------------- |
| `src/`                           | Rust backend: Axum routes, DB, metadata, jobs, organize                       |
| `src/lib/`                       | Rust library code (DB, utilities, shared logic)                               |
| `src/bin/`                       | Rust binary entry points (e.g., `main.rs`)                                    |
| `src/migration.rs`               | Main migration runner + Rust migration modules (`src/migration/*.rs`)         |
| `frontend/src/`                  | Svelte 5 frontend source                                                      |
| `frontend/src/lib/bindings/*.ts` | ⚠️ **AUTO-GENERATED** — sync types from Rust `#[derive(TS)]` — **do not edit** |
| `frontend/src/lib/components/`   | Svelte components                                                             |
| `frontend/src/lib/state/`        | Svelte 5 state management                                                     |
| `frontend/src/pages/`            | Page routes                                                                   |
| `migrations/`                    | sqlx SQLite migrations                                                        |
| `templates/`                     | Handlebars config templates                                                   |
| `frontend/plugins/icons.ts`      | Custom Vite plugin for bundling icons                                         |
| `.sqlx/`                         | Cached SQL queries (created during build, used for offline mode)              |

---

## Database

- **SQLite only** (no PostgreSQL/MySQL)
- **Offline mode enabled**: `SQLX_OFFLINE=true` in `.cargo/config.toml`
- No live DB needed at compile time; `.sqlx/` caches query results
- **New migration**: `sqlx migrate add -r <name>` (requires `sqlx-cli` + live DB connection)
- **Custom migrations**: Data migrations live in `src/migration/*.rs` with a `VERSION` constant and `migrate(pool: &SqlitePool)` function, registered in `src/migration.rs`

---

## Architecture & Patterns

### Build Flow

1. `cargo build` triggers `build.rs`
2. `build.rs` runs `pnpm build` → generates `frontend/dist/`
3. `rust-embed` embeds `dist/` into binary at compile time
4. Backend serves frontend statically at runtime

### Config System

- **File**: `config.toml` (TOML format)
- **Template**: Uses Handlebars `{{ }}` syntax for interpolation
- **Env overrides**: `MUUSIK_CONFIG_DIR`, `MUUSIK_DATA_DIR`, `MUUSIK_CACHE_DIR`

### Real-time Updates

- SSE endpoint: `/api/events`
- Emits: job progress, file operations, metadata changes

### Svelte 5 Specifics

- Uses `mount()` API (not `new App()`)
- All components use **runes syntax** (`$state`, `$derived`, etc.)

---

## Formatting Rules

### dprint Configuration (`dprint.json`)

**File types formatted by dprint:**

- TypeScript (`.ts`, `.tsx`)
- Markdown (`.md`, `.markdown`)
- TOML (`.toml`)
- Markup (`.svelte` — uses `markup_fmt` plugin)
- Malva (`.mjs`, `.cjs` — config at `malva` key)

**Excluded from dprint:**

```json
["**/node_modules", "**/*-lock.json", "frontend/src/lib/bindings/*", "*.svg"]
```

**Plugin sources (from `dprint.json`):**

```json
[
  "https://plugins.dprint.dev/typescript-0.95.15.wasm",
  "https://plugins.dprint.dev/markdown-0.21.1.wasm",
  "https://plugins.dprint.dev/g-plane/malva-v0.15.2.wasm",
  "https://plugins.dprint.dev/g-plane/markup_fmt-v0.26.0.wasm",
  "https://plugins.dprint.dev/toml-0.7.0.wasm"
]
```

**Key settings:**

- `useTabs: true` — **always use tabs** (not spaces)
- `markup.scriptIndent: true`, `styleIndent: true` — indent `<script>` and `<style>` in Svelte

### Prettier

- Only used for **`.svelte` files** (dprint's markup_fmt is used for the rest)

---

## Code Conventions

### Commits

- Use **Conventional Commits** format (e.g., `feat: add metadata scanner`, `fix: resolve DB deadlock`)

### Rust

- Edition: **2024** (not 2021)
- Use `tracing` for logging (not `println!`)
- Error handling: prefer `thiserror` + `anyhow`

### TypeScript

- Path aliases: `@api/*`, `@components/*`, `@lib/*`, `@pages/*`, `@utils/*`, `@assets/*`, `@attachments/*`, `@state`
- Type safety: use strict mode, avoid `any`
- Svelte 5: use runes (`$state()`, `$derived()`, `$effect()`)

### Backend Types → Frontend Types

- Rust structs use `#[derive(TS)]` (via `ts-rs`)
- Types sync automatically to `frontend/src/lib/bindings/*.ts`
- **Never edit bindings manually**

---

## Common Tasks

### Adding a new migration

**Two-part migration system**:

1. **SQL Migration** (for schema changes):
   ```bash
   sqlx migrate add -r <name>  # requires live DB connection
   ```
   - SQL file created in `migrations/<timestamp>_<name>.sql`
   - Handles `CREATE TABLE`, `ALTER TABLE`, indexes, etc.
   - Automatically applied by `sqlx::migrate!()` macro

2. **Custom Rust Migration** (for data migrations, complex logic):
   - Create file: `src/migration/<migration_name>.rs`
   - Required exports:
     ```rust
     pub const VERSION: i64 = <timestamp_like_i64>;  // e.g., 20260822000717

     pub async fn migrate(pool: &SqlitePool) -> Result<()> {
         // Your migration logic here
         // Use `query!`, `query_as!`, transactions, etc.
         // Log progress with `info!("...")`
     }
     ```
   - Register in `src/migration.rs`:
     ```rust
     static MIGRATIONS: LazyLock<HashMap<i64, MigrationFn>> = migration_map! {
         add_uuid_to_songs,  // your module name
         // ... other migrations
     };
     ```
   - **Note**: Rust migrations run AFTER the SQL migration is applied (as "after-logic")

3. **Run migrations**:
   ```bash
   cargo run  # auto-runs on startup
   ```

**Pattern for Rust migrations**:

- Always use transactions (`pool.begin().await?`)
- Check for empty data early and return `Ok(())`
- Use `info!()` for progress logging with timings
- Validate `rows_affected()` after writes
- Use `time::OffsetDateTime::now_utc()` for timestamps
- Write tests using `apply_migrations()` helper

### Adding a new API endpoint

1. Define route in `src/routes.rs` (or appropriate module)
2. Add handler function
3. Update frontend API types (if needed) → run `cargo build` to sync bindings

### Updating Svelte component

1. Use runes syntax (`<script>` with `$state`, etc.)
2. Format with `dprint fmt` (markup_fmt plugin handles `.svelte`)
3. Run `pnpm test` to verify

---

## Troubleshooting

| Issue                               | Solution                                                                           |
| ----------------------------------- | ---------------------------------------------------------------------------------- |
| `cargo build` fails with SQLX error | Run `cargo build --features sqlx-offline` or ensure `.sqlx/` exists                |
| Frontend not loading in binary      | Check `build.rs` ran `pnpm build` and `dist/` is non-empty                         |
| Type mismatch between Rust ↔ TS     | Run `cargo build` — `ts-rs` will regenerate bindings                               |
| Dprint format errors                | Run `dprint check` first to see what's failing, then `dprint fmt`                  |
| Prettier errors on `.svelte`        | Ensure Prettier is installed; dprint's markup_fmt should handle most cases         |
| Vite proxy not working              | Confirm backend on `:3000`, Vite on `:5173`, check `vite.config.ts` proxy settings |

---

## Environment Variables

| Variable            | Description                  | Default                          |
| ------------------- | ---------------------------- | -------------------------------- |
| `MUUSIK_CONFIG_DIR` | Path to config directory     | `~/.config/muusik`               |
| `MUUSIK_DATA_DIR`   | Path to data directory       | `~/.local/share/muusik`          |
| `MUUSIK_CACHE_DIR`  | Path to cache directory      | `~/.cache/muusik`                |
| `SQLX_OFFLINE`      | Enable offline mode for sqlx | `true` (in `.cargo/config.toml`) |

---

## Git Hooks (Future)

- Pre-commit: run `dprint fmt` + `dprint check`
- Pre-push: run `cargo test` + `pnpm test`
