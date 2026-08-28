# Seer (Rust)

OSINT platform on [lariv-rs](../../lariv-rs): intel ingest, node scraper fleet, website/Reddit/Twitter sources, GDELT/OpenSky/AIS stream connectors, and LLM assistant tools.

Full parity port of Go [`deployments/seer`](../seer).

## Workspace crates

| Crate | Role |
|-------|------|
| `seer-bin` | Binary `seer-rs` — installs all plugins |
| `seer-common` | Shared worker helpers / categories |
| `seer-workerregistry` | Worker run logs + active workers UI |
| `seer-intel` | Intel + events, embeddings, vector search (pgvector) |
| `seer-node-fleet` | Scraper WebSocket fleet (`/fleet/websocket`) |
| `seer-websites` | Crawl/scrape via fleet + SSRF checks |
| `seer-reddit` | Reddit RSS ingest |
| `seer-twitter` | Nitter RSS ingest |
| `seer-gdelt` | BigQuery GDELT worker |
| `seer-opensky` | OpenSky OAuth + states poller |
| `seer-aisstream` | aisstream.io WebSocket client |
| `seer-assistant` | LLM tools (`intel_search`, `reddit_*`, `website_*`) |
| `nodescraper-rs` | Edge scraper (Servo + prost); nested Cargo workspace (sqlite `links` conflict with sea-orm) |

## Quick start

```bash
# Postgres recommended for pgvector intel search
# edit config.toml: database_url, admin user, etc.

cargo run -p seer-bin
```

Nightly Rust is required (`impl_trait_in_assoc_type`). Pin used in CI: `nightly-2026-08-11`.

## Config

See [`config.toml`](config.toml) for core Lariv sections (`database_url`, `users`, `filesystem`, `pwa`).

Per-app API keys and connector settings live in each app’s **Preferences** page (staff):

| App | Path |
|-----|------|
| Intel | `/seer-intel/preferences` |
| Twitter | `/seer-twitter/preferences` |
| AIS Stream | `/seer-aisstream/preferences` |
| OpenSky | `/seer-opensky/preferences` |
| GDELT | `/seer-gdelt/preferences` |

Env overrides: `DATABASE_URL`, `BIND`, `RUST_LOG`, `MAP_STYLE_URL` (optional Intel Map [MapLibre GL JS](https://maplibre.org/maplibre-gl-js/docs/) style; defaults to OpenFreeMap Liberty). LLM keys are set in the LLM Assistant preferences UI.

## Fleet scrapers (`nodescraper-rs`)

Vendored from Go Seer; shares [`proto_src/scraper.proto`](proto_src/scraper.proto). Nested workspace because Servo’s rusqlite and lariv’s sqlx-sqlite both claim `links = "sqlite3"`.

```bash
cargo build --release --manifest-path nodescraper-rs/Cargo.toml

# Default WS URL is ws://localhost:42069/fleet/websocket
REMOTE_URL=wss://seer.lariv.in/fleet/websocket cargo build --release --manifest-path nodescraper-rs/Cargo.toml
```

## Ops notes

- For vector intel search, install pgvector and enable it **before** (or after, with a column alter) migrate:
  ```sql
  CREATE EXTENSION IF NOT EXISTS vector;
  ```
  The intel migration does **not** run `CREATE EXTENSION` itself (a failure would abort the migration transaction). If the `vector` type is missing, `seer_intels.embedding` is created as `TEXT`.
- Run at least one `nodescraper-rs` node before website crawls.
- GDELT uses Application Default Credentials (`GOOGLE_APPLICATION_CREDENTIALS`) plus `projectId` from GDELT Preferences.

## CI / release

- `.github/workflows/ci.yml` — `cargo check --release` (default members) + mount smoke test
- `.github/workflows/release.yml` on `v*` tags — uploads `seer-rs` and `nodescraper-rs`
