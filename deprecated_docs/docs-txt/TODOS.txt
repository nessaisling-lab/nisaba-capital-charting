# TODOS

## docker-compose.yml for local PostgreSQL

**What:** Add a `docker-compose.yml` to the repo root that starts a PostgreSQL instance.
**Why:** "Install PostgreSQL" is a 30-minute Windows adventure. `docker compose up -d` is 30 seconds and reproducible.
**Pros:** Reproducible setup, no system PostgreSQL needed, works on instructor's machine for demo.
**Cons:** Requires Docker Desktop installed.
**Context:** 10-line file. Would set `POSTGRES_PASSWORD=dev`, `POSTGRES_DB=financial_dashboard`, expose port 5432. The DATABASE_URL in `.env.example` already matches this config.
**Blocked by:** Nothing — can be added any time before the demo.
