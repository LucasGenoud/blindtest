# Blindtest

## AI Slop Warning
This repository was generated and migrated using an AI assistant. While the code functions as intended, some sections may contain sub‑optimal patterns or leftover artefacts from the migration process. Review and test thoroughly before using in production.

## Project Overview
- **Original stack**: Vue.js (frontend) and Node.js (backend).
- **Current stack**: SvelteKit for the frontend and Rust for the backend services.
- **Migration tool**: Claude Opus 4.6 was used to translate the original codebase to the new technologies.
- **Containerisation**: Docker Compose files (`docker-compose.yml` and `docker-compose.prod.yml`) define development and production environments.
- **Data migration**: Python scripts (`migrate.py`, `migrate_videos.py`) handle migration of assets to S3 storage.
- **Key directories**:
  - `client/` – SvelteKit source code.
  - `server/` – Rust backend source code.
  - `restore/` – scripts for restoring previous state.
  - `docker‑compose*.yml` – Docker configuration.

## Features
- Manage audio assets with S3 integration and playback directly from the dashboard.
- Migrate existing video files to S3 using Python migration scripts.
- Real‑time audio upload and processing via Rust backend routes.
- User authentication and cursor visibility handling across canvas interactions.
- Docker‑compose based development and production environments.
- API endpoints for audio CRUD operations with robust error handling.
- Configurable via environment variables for local and production setups.

## Getting Started
```bash
# Build and run the development environment
docker compose up -d --build
```

Refer to the individual `README` files in `client/` and `server/` for detailed setup instructions.

## License
[Specify your license here]
