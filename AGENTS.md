# Blindtest – Agent Instructions

## Architecture

- **Monorepo** with two packages:
  - `client/` — SvelteKit 5 (runes mode), Tailwind CSS v4, adapter-node
  - `server/` — Rust, Actix-web 4, SQLite (`rusqlite`), JWT auth via PEM keys
- **Storage**: SeaweedFS (S3-compatible). Docker dev compose includes a `seaweedfs` service.
- **Database**: SQLite at `server/data/blindtest.db` (gitignored, created on server start).
- **Realtime**: WebSocket endpoint at `/ws`, uses a `WsBroadcaster` shared via `web::Data`.

## Commands

There are no lint, test, or typecheck scripts in either package. Verification is manual.

### Full stack (Docker — preferred)

```bash
# Copy .env.example → .env and fill required values, then:
docker compose up -d --build
```

Services and ports:
- `seaweedfs` — S3 API on `:8333`, admin on `:9333`
- `server` — Actix on host `:3080` → container `:80`
- `client` — SvelteKit on host `:3000` → container `:3000`

### Client only (native)

```bash
cd client && npm install
npm run dev       # vite dev server
npm run build     # production build → build/
npm run preview   # preview production build
```

### Server only (native)

```bash
cd server && cargo run
```

Requires `server/secret/private.pem` and `server/secret/public.pem` to exist.

### Production deploy

Use `docker-compose.prod.yml`. It connects to the external `pangolin` network instead of the default bridge.

### Data migration (one-off)

- `python3 migrate.py` — imports JSON dumps from `restore/` into `server/data/blindtest.db`
- `python3 migrate_videos.py` — uploads video/audio files to SeaweedFS S3

## Environment Variables

See `.env.example` for the full list. Key ones:

| Variable | Used By | Notes |
|---|---|---|
| `VITE_API_URL` | client (build-time) | Backend HTTP URL, baked into JS. Set via Docker build arg. |
| `VITE_WS_URL` | client (build-time) | Backend WebSocket URL, baked into JS. Set via Docker build arg. |
| `FRONTEND_URL` | server (runtime) | Comma-separated CORS origins. Unset = allow all (dev-safe, prod-unsafe). |
| `CLIENT_ORIGIN` | client (runtime) | SvelteKit CSRF/origin validation. |
| `SERVER_S3_*` | server (runtime) | SeaweedFS/AWS S3 connection details. |
| `LLM_BASE_URL`, `LLM_MODEL` | server (runtime) | OpenAI-compatible endpoint for the blindtest assistant. Both unset = feature disabled and hidden in the client. |
| `LLM_API_KEY` | server (runtime) | Bearer token for that endpoint. Optional — local servers often need none. |
| `LLM_CONTEXT_TOKENS` | server (runtime) | Context window the endpoint serves, not what the model claims (default 32768). The catalog is trimmed to fit it. |
| `LLM_EXTRA_BODY` | server (runtime) | JSON object merged into every request body. Provider-specific switches, e.g. `{"chat_template_kwargs":{"enable_thinking":false}}` to stop a reasoning model thinking. |

## Server Quirks

- **Migrations** are embedded SQL files (`server/migrations/*.sql`), applied at startup via `include_str!` in `db.rs`. Do not run them manually; restarting the server applies them (idempotent enough for ALTER TABLE).
- **DB path**: `data/blindtest.db` relative to the working directory. In Docker, this is `/app/data/blindtest.db` → mounted to `./server/data/`.
- **Canvas**: 1000×1000 grid (1M rows in `canvas_pixels`). Initialized on first startup — takes time.
- **Auth**: JWT signed with PEM key pair at `server/secret/`. Server mounts these read-only.
- **Video processing**: spawns `yt-dlp` and `ffmpeg` on the host/container. Both must be installed (Dockerfile handles this).
- **Blindtest assistant** (`src/llm.rs`, `src/routes/blindtest_agent.rs`): talks to any OpenAI-compatible
  `/chat/completions`. The whole playable library is put in the prompt as a numbered catalog and the
  model answers with catalog numbers, never titles, so a generated blindtest is always playable. One
  conversation thread per blindtest lives in `blindtest_agent_messages`. Rate limited to 30 prompts
  per account per hour. `POST /streamblindtest/{id}` returns SSE and is what the client uses;
  `POST /generateblindtest/{id}` does the same work in one response and is the fallback for proxies
  that buffer `text/event-stream`. Both share `prepare` / `interpret` / `persist`.

## Client Quirks

- Uses `@sveltejs/adapter-node` — builds to a Node.js server, not static files.
- Runes mode is forced in `svelte.config.js` for Svelte 5.
- API calls go through `client/src/lib/api.js` — all requests target `$env.public.VITE_API_URL`.
- WebSocket calls go through `client/src/lib/websocket.js` — connects to `$env.public.VITE_WS_URL`.
