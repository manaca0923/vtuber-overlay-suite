# Repository Guidelines

This repo contains a Tauri desktop app for VTuber streaming tools. The frontend is React + TypeScript (Vite) and the backend is Rust (Tauri, Axum, SQLite).

## Project Structure & Module Organization
- `src/`: React app entry (`main.tsx`, `App.tsx`), UI components in `src/components/`, shared types in `src/types/`, utilities in `src/utils/`, and static assets in `src/assets/`.
- `src-tauri/`: Rust backend, Tauri config, and local overlays. Core code lives in `src-tauri/src/` with commands, YouTube polling, WebSocket/HTTP server, and DB logic; migrations and sample DBs live in `src-tauri/migrations/` and `src-tauri/dev.db`.
- `src-tauri/overlays/`: HTML/CSS/JS used by OBS browser sources.
- `docs/`: Design and architecture references; start with `docs/100_architecture.md` and `docs/110_development-environment.md`.

## Build, Test, and Development Commands
- `npm install`: install frontend dependencies.
- `npm run dev`: Vite dev server for frontend only.
- `npm run tauri:dev`: run the full desktop app in dev mode.
- `npm run build`: typecheck and build frontend.
- `npm run tauri:build`: build a release desktop app.
- `npm run typecheck`: TypeScript project checks.
- `npm run lint`: run ESLint.
- `cargo test` (from `src-tauri/`): run Rust unit tests.

## Coding Style & Naming Conventions
- TypeScript/React uses 2-space indentation, semicolons, and single quotes; keep components in PascalCase (e.g., `SetlistForm.tsx`) and hooks prefixed with `use`.
- Rust follows standard `rustfmt` style; keep modules small and grouped by feature (commands, youtube, server, db).
- Run `npm run lint` before pushing.

## Testing Guidelines
- Rust tests are co-located in modules using `#[cfg(test)]` and run via `cargo test`.
- No frontend test runner is configured; if adding one, document it and include a script in `package.json`.
- Prefer small, deterministic unit tests for parsers and state helpers.

## Commit & Pull Request Guidelines
- Commit messages follow Conventional Commits (e.g., `feat:`, `fix:`, `docs:`, `refactor:`, `test:`).
- Branch names use `feature/<topic>-<short-scope>` (e.g., `feature/youtube-api-polling`).
- PRs should include a short summary, linked issue/task, and screenshots for UI changes. After opening a PR, add an `@codex review` comment and re-run it after fixes.
- Update `docs/900_tasks.md` checklists when work is completed.

## Security & Configuration Tips
- API keys are stored in OS keyring; never log or commit secrets.
- Local servers default to `localhost:19800` (HTTP) and `localhost:19801` (WebSocket).
