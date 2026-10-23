# Muusik Frontend Agent Guide

## Tech Stack

- **Framework**: Svelte 5 (SvelteKit 3 for routing/SSR, runes mode)
- **Language**: TypeScript 6+ (strict mode)
- **Styling**: Tailwind CSS 4 + custom CSS
- **Build Tool**: Vite
- **Testing**: Vitest + Playwright (browser tests) + Testing Library
- **Package Manager**: pnpm (configured via `pnpm-workspace.yaml`)
- **Formatters/Linters**: `dprint` (formatting), `oxlint` (linting)

## Commands

```bash
# Development
pnpm dev              # Vite dev server on :5173, proxies /api → localhost:3000
pnpm build            # Build for production (creates dist/)

# Testing
pnpm test             # Run all tests (client + server)
pnpm test:unit        # Run unit tests only
pnpm check            # TypeScript + Svelte type checking
pnpm check:watch      # Watch mode for type checking

# Code Quality
pnpm exec oxlint       # Frontend linting
dprint check           # Format check (from root)
dprint fmt             # Auto-format (from root)
```

**MAKE SURE TO RUN PNPM RUN CHECK BEFORE YOU ARE FINISHED WITH YOUR CHANGES!**

## Path Aliases (`package.json` → `imports`)

| Alias            | Resolves To                      |
| ---------------- | -------------------------------- |
| `#components/*`  | `./src/lib/components/*`         |
| `#assets/*`      | `./src/lib/assets/*`             |
| `#api/*`         | `./src/lib/api/*.ts`             |
| `#utils/*`       | `./src/lib/utils/*.ts`           |
| `#attachments/*` | `./src/lib/attachments/*.ts`     |
| `#bindings/*`    | `./src/lib/bindings/*.ts`        |
| `#bindings`      | `./src/lib/bindings/bindings.ts` |
| `#lib/*`         | `./src/lib/*`                    |

**Important**: These aliases work with SvelteKit's import resolution. Use them consistently.

## Project Structure

```
frontend/src/
├── lib/                    # Library code
│   ├── api/               # API client functions
│   │   ├── albums.ts      # Album API calls
│   │   ├── directory.ts   # Directory API calls
│   │   ├── jobs.ts        # Job status API
│   │   └── song.ts        # Song metadata API
│   ├── assets/            # Static assets (icons, images)
│   ├── attachments/       # File attachment utilities
│   │   ├── mutation.ts    # File mutation helpers
│   │   ├── resize.ts      # Image resizing
│   │   └── viewport.ts    # Viewport-aware loading
│   ├── bindings/          # Auto-generated Rust → TS types (NEVER EDIT)
│   ├── components/        # Reusable Svelte components
│   │   ├── *.svelte       # Base UI components (Button, Modal, etc.)
│   │   ├── input/         # Input components
│   │   ├── music/         # Music-specific components
│   │   ├── navigation/    # Nav components
│   │   ├── stack/         # Stack layout components
│   │   └── table/         # Table components
│   ├── stores/            # Svelte state stores
│   ├── utils/             # Utility functions
│   │   ├── api.ts         # Fetch wrapper
│   │   ├── reactivity/    # Svelte reactivity helpers
├── routes/                # SvelteKit pages/routes
│   ├── +layout.svelte     # Root layout
│   ├── +layout.ts         # Layout load function
│   ├── +page.ts           # Home page load
│   ├── albums/            # Album list + detail routes
│   ├── settings/          # Settings pages
│   └── layout.css         # Global styles
└── app.html               # HTML template
```

## SvelteKit 3 Notes

- configuration now lives in vite.config.ts instead of svelte.config.js
- No longer use $lib alias as it's been replaced now #lib, making use of standard subpath imports in package.json

It's very important to run `pnpm check` before committing changes to ensure there are no type errors;

## Svelte 5 Patterns Used

### State Management (Svelte Stores)

```svelte
<!-- Using $state for component-local state -->
<script lang="ts">
	let count = $state(0);
</script>

<!-- Using classes with $state for global stores -->
<script lang="ts">
	export class AppState {
		#songs = $state<Map<string, DatabaseSong>>(new Map());
		#selectedSongs = new SvelteSet<string>();

		async refreshSongs() {
			this.#songs = new Map(await getSongs());
		}
	}
</script>

<!-- Using $derived for computed values -->
#albumSongIdMap = $derived.by(() => {/* complex derivation */});

<!-- Using $effect for side effects (optional debug logging) -->
$effect.root(() => {$inspect(this.#albums, this.#albumSongIdMap)});
```

### Component Imports

```svelte
<!-- Import components using #components alias -->
<script lang="ts">
	import Button from "#components/Button.svelte";
	import Card from "#components/Card.svelte";
	import Table from "#components/table/Table.svelte";
</script>

<!-- Import base UI components with proper structure -->
<!-- Button.svelte, TextInput.svelte, Modal.svelte, etc. -->
```

### API Layer Pattern

```typescript
// src/lib/api/song.ts
import type { DatabaseSong } from "#bindings/DatabaseSong";
import { fetchJson } from "#utils/api";

export async function getSongs(): Promise<Array<DatabaseSong>> {
	return await fetchJson<Array<DatabaseSong>>("/api/songs/");
}

export async function getSongFileInfo(id: string): Promise<SongFile> {
	return await fetchJson<SongFile>(`/api/songs/${id}/file-info`);
}

export async function editSong(id: string, song: SongMetadata): Promise<void> {
	await fetchJson<void>(`/api/songs/${id}`, {
		method: "PUT",
		body: JSON.stringify(song),
	});
}
```

### Type Composition

```typescript
// models.ts combines DB types + metadata types
import type { DatabaseSong } from "#bindings/DatabaseSong";
import type { SongMetadata } from "#bindings/SongMetadata";

export type Song = DatabaseSong & SongMetadata;
```

## Key Components Reference

| Component          | Location                          | Purpose                                |
| ------------------ | --------------------------------- | -------------------------------------- |
| `Button.svelte`    | `#components/Button`              | Primary action button                  |
| `TextInput.svelte` | `#components/TextInput`           | Text input with validation             |
| `Modal.svelte`     | `#components/Modal`               | Dialog/modal overlay                   |
| `Table.svelte`     | `#components/table/Table`         | Data table with `@tanstack/table-core` |
| `GridList.svelte`  | `#components/GridList`            | Grid/list view toggle                  |
| `CoverView.svelte` | `#components/music/CoverView`     | Album art display                      |
| `Editor.svelte`    | `#components/music/Editor`        | Song metadata editor                   |
| `PrevPage.svelte`  | `#components/navigation/PrevPage` | Navigation history                     |

## State Stores

### AppState (Global)

```typescript
import { app } from "#stores/app";

// Usage in Svelte components:
app.songs; // Map<string, DatabaseSong>
app.albums; // Map<string, Album>
app.selectedSongs; // SvelteSet<string>
app.editedSongs; // SvelteMap<string, Song>

// Trigger data refresh:
await app.refreshSongs();
await app.refreshAlbums();
```

### Input Store

```typescript
import { input } from "#stores/input";
// For form/input state management
```

## Testing

### Vitest Configuration

- Two test projects: `client` (browser/Playwright) and `server` (Node)
- Client tests: `src/**/*.svelte.{test,spec}.{js,ts}`
- Server tests: `src/**/*.{test,spec}.{js,ts}` (excludes `.svelte`)

```typescript
// Example test file: MyComponent.test.svelte
import { render, screen } from "@testing-library/svelte";
import MyComponent from "./MyComponent.svelte";

test("displays expected content", () => {
	render(MyComponent);
	expect(screen.getByText("Expected")).toBeInTheDocument();
});
```

### API Testing Utilities

```typescript
import { fetchJson } from "#utils/api";

// Custom fetch wrapper in #utils/api.ts
```

## Styling

### Tailwind CSS 4

- Uses new CSS-first configuration
- Import in layout: `@import "tailwindcss";`
- Custom styles in `routes/layout.css`

### Custom Components

- Components use Tailwind classes + custom CSS where needed
- Icons from `@iconify-json/mingcute` via `@iconify/svelte`
- Custom font: JetBrains Mono (variable) + Inter (variable)

## Development Patterns

### Adding a New Page

1. Create route in `routes/` (e.g., `routes/my-page/+page.svelte`)
2. Add load function if needed (`+page.ts`)
3. Use Svelte 5 runes: `let data = $state(null);`
4. Call API via `#api/*` imports
5. Add store state updates in `#stores/`

### Adding a New Component

1. Create in `src/lib/components/`
2. Use existing component as pattern (e.g., `Button.svelte`)
3. Export with clear props using Svelte 5 runes
4. Add tests if complex

### Adding New API Endpoint

1. Add function to appropriate `src/lib/api/*.ts` file
2. Use `fetchJson` wrapper for type safety
3. Import in components: `import { myApiFunction } from '#api/my-module'`

### Using Bindings (Auto-Generated Types)

- Types live in `src/lib/bindings/`
- Generated from Rust `#[derive(TS)]` attributes
- **NEVER EDIT THESE FILES** - regenerate by running backend build
- Import: `import type { SongMetadata } from '#bindings/SongMetadata'`

## Common Tasks

### Fetch Data from Backend

```typescript
import { getSongs } from "#api/song";
import { app } from "#stores/app";

// Via store (preferred for global data)
await app.refreshSongs();
const songs = app.songs;

// Direct API call
const songs = await getSongs();
```

### Update Song Metadata

```typescript
import { editSong } from "#api/song";
import type { SongMetadata } from "#bindings/SongMetadata";

await editSong(songId, {
	title: "New Title",
	artist: "New Artist",
	// ... other fields
});
```

## Important Notes

1. **Never edit** `src/lib/bindings/*.ts` - these are auto-generated from Rust
2. **Use path aliases** consistently (`#components/`, `#api/`, etc.)
3. **Svelte 5 runes** are required - no legacy syntax
4. **Store updates** should be done through store methods, not direct mutation
5. **Proxy configuration**: Vite dev proxies `/api/*` to `localhost:3000`
6. **TypeScript strict** mode is enforced - avoid `any` where possible
7. **Format with dprint** before committing (tabs, specific config in root)

## Debugging

- Use `$inspect()` in `$effect.root()` to debug store changes
- Check Vitest output for test failures
- Network tab for API calls (check proxy)
- SSE events stream at `/api/events` for real-time updates

## MCP Servers

You are able to use the Svelte MCP server, where you have access to comprehensive Svelte 5 and SvelteKit documentation. Here's how to use the available tools effectively:

### Available Svelte MCP Tools:

#### 1. list-sections

Use this FIRST to discover all available documentation sections. Returns a structured list with titles, use_cases, and paths.
When asked about Svelte or SvelteKit topics, ALWAYS use this tool at the start of the chat to find relevant sections.

#### 2. get-documentation

Retrieves full documentation content for specific sections. Accepts single or multiple sections.
After calling the list-sections tool, you MUST analyze the returned documentation sections (especially the use_cases field) and then use the get-documentation tool to fetch ALL documentation sections that are relevant for the user's task.

#### 3. svelte-autofixer

Analyzes Svelte code and returns issues and suggestions.
You MUST use this tool whenever writing Svelte code before sending it to the user. Keep calling it until no issues or suggestions are returned.

#### 4. playground-link

Generates a Svelte Playground link with the provided code.
After completing the code, ask the user if they want a playground link. Only call this tool after user confirmation and NEVER if code was written to files in their project.

## References

- [Svelte 5 Docs](https://svelte.dev/docs/svelte)
- [SvelteKit Docs](https://kit.svelte.dev/docs)
- [Tailwind CSS 4](https://tailwindcss.com/docs)
- [Vitest Docs](https://vitest.dev)
- [Dprint](https://dprint.dev)
- [Oxlint](https://oxlint.rs)
