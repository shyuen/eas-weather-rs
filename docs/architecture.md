# EAS Ingest & Delivery Architecture

Design notes for the emergency alert system (EAS) microservice. This covers the
planned **listener** (ingest) application and how it complements the existing
read-only query service, as well as the resilience, dedup, filtering, and
delivery design.

## Overview

Two applications, split by lifecycle (plus a planned auth microservice):

1. **`eas-weather-rs`** (this crate) — a read-only query API serving alert data
   over HTTP from a database.
2. **Listener (new, separate)** — a long-running consumer that receives alert
   data over a socket connection (Plemorex), validates, archives, deduplicates,
   filters, and delivers alerts to clients. It never serves HTTP requests.
3. **Auth microservice (planned)** — a JWT *issuer* for a future user portal and
   service-to-service auth; see [Authentication](#authentication).

The split keeps two very different lifecycles apart: a request/response server
vs. a long-running subscriber with a live socket.

## Data flow

```
Plemorex feed
     │
     ▼
[Listener replica]  (many, any region)
     │
     ├─► validate (parse into domain Alert newtypes)
     │
     ├─► archive raw to object storage (bucket)   [durable, append-only]
     │
     ├─► pre-filter against Redis "seen" set      [hot tracking / efficiency]
     │
     └─► persist to DB (idempotent upsert)        [source of record]
              │
              ▼
        [region filter]  ─────────────────────────►  deliver to clients
```

## Components and their roles

| Tier | Role | Durability | Dedup role |
|------|------|-----------|------------|
| Plemorex feed | live source | — | — |
| Listener buffer / queue | capture while DB down | durable-ish | prevents loss |
| Redis | recent tracking, fast pre-filter, client ack state | volatile | efficiency (avoid redundant work) |
| Object storage (bucket) | raw immutable archive + replay source | durable | idempotent object keys |
| DB (e.g. Spanner / CockroachDB) | source of record for the query service | durable | correctness via unique index |
| Clients | local display tracking | client-side | continuity |

## Key design decisions

### Separate listener application
The listener is a separate application from `eas-weather-rs`. Mixing a live
socket consumer into the request/response server would couple two incompatible
lifecycles.

### Single globally-consistent database
All replicas write to one logical database with **global uniqueness**. With a
multi-region, strongly-consistent store (Spanner, CockroachDB), physical
replication across regions still yields one logical DB. A **unique index on
`identifier`** is the single, authoritative dedup point — regardless of how many
listener replicas or regions exist.

- Correctness relies on the DB constraint + **idempotent upsert**
  (`INSERT ... ON CONFLICT DO NOTHING` / `ON DUPLICATE KEY UPDATE`), never a
  racy check-then-insert.
- Replicas are interchangeable publishers; the DB is the authority.

### Deduplication key
The CAP **`identifier`** field is the natural uniqueness key. Enforce it in
layers:

- **DB unique index** — the hard correctness guarantee (handles races and
  cross-replica contention).
- **Idempotent upsert** — removes the need for a racy read-before-write.
- **Redis "seen" set** — optional hot pre-filter for efficiency only, never the
  correctness authority (Redis is volatile).

If the feed ever violates `identifier`'s global uniqueness, dedup on
`(sender, identifier)` or the composite `ExtendedMessageIdentifier`.

### Multi-region replicas
Multiple listener replicas across regions may each receive the same alert.
With a single globally-consistent DB, concurrent duplicate inserts collapse
into normal DB concurrency: one wins, the rest no-op. No coordination layer or
queue is required *for correctness*.

A queue/consumer-group is only warranted if **avoiding redundant work** matters
(each replica parsing/inserting every alert). In that case use a keyed topic +
consumer group (same `identifier` → same partition → one replica owns it). A
peer-to-peer "chat"/gossip layer between replicas is **not** recommended — it
has its own consistency/split-brain problems and duplicates what the DB already
provides.

### Resilience: DB offline temporarily
The DB is not the only record of what happened (clients track what they've
received/shown). Tiered resilience:

1. **Listener durable buffer/queue** — prevents loss; the DB write is
   downstream and retried. Replay is idempotent via the unique index.
2. **Client local tracking + live fan-out** — keeps service continuity; the
   listener fans out new alerts directly to connected clients, which cache and
   display them locally.
3. **Reconciliation on reconnect** — restore the DB as source of record using
   the reliable listener/bucket records (client backfill is best-effort only).

The **listener buffer/bucket is the reliable recovery source**; clients are a
best-effort second source (some may be offline or cleared).

### Redis and object storage
- **Redis** = hot, in-memory tracking layer: recent-`identifier` seen-set,
  dedup pre-filter, client ack state, per-region last-sent state. Volatile —
  never the source of truth.
- **Object storage (bucket)** = warm/cold durable archive: every raw CAP message
  keyed by `identifier`/timestamp. Immutable keys make archiving idempotent
  (same key = same object). This is the ultimate replay/backfill source if the
  DB or Redis ever loses data.

### Region / severity filtering
Filtering decides **what is sent**, not **what is stored**:

- **Store everything** (full archive in DB + bucket).
- **Filter on the send/serve path** (region, severity, certainty, urgency).

This keeps the archive complete and the filter reversible — changing a region's
boundary or adding a region never requires replaying history.

CAP provides the region data in the alert itself: `<scope>`, `<area>` blocks
with `<geocode>` (FIPS/`SAME` codes), and `<polygon>`/`<circle>` coordinates.
A region filter is therefore a decidable function of the alert's own fields.

Model filtering as a **rule engine** (a `FilterPort`), configurable per region,
not hardcoded:
- geocode/`SAME` intersection,
- geometric overlap (point-in-polygon / bounding-box pre-check),
- severity/certainty/urgency thresholds.

## Pipeline per listener replica

1. **Receive** from Plemorex → validate into `Alert` newtypes.
2. **Archive raw** to the bucket (immutable, keyed by `identifier`).
3. **Pre-filter** against Redis "seen" set → skip if already processed.
4. **Persist** to the DB with an idempotent upsert (unique index).
5. **Filter** (region/severity) and **deliver** to clients; track client acks in
   Redis.

## Suggested hexagonal ports (listener)

- `FeedPort` — socket/subscription to Plemorex.
- `ArchivePort` — durable raw-alert object storage.
- `TrackingPort` — Redis seen-set / ack state.
- `StorePort` — idempotent persistence to the DB.
- `FilterPort` — region/severity rule decisions.
- `DeliveryPort` — fan-out to clients.

## Authentication

Two distinct consumers authenticate against the API:

1. **DEV / internal access** to `eas-weather-rs` endpoints (`/alerts/*`, `/meta/*`).
2. **End users** of a future **user portal** (manual alert viewing/sending).

Both can share a single **issuer**: a dedicated **auth microservice** that handles
login, credentials, roles/scopes, and **mints JWTs**. Validation of JWTs is
stateless and stays embedded in each consuming service; only issuance lives in
the auth microservice.

```
[User Portal] ──login──▶ [Auth microservice] ──issues JWT──┐
                                                          │  shared key
[DEV / other svc] ──login──▶ [Auth] ──JWT─────────────────┼──▶ [eas-weather-rs] validates
                                                          │  (JWK)
                                                          └──▶ [listener] validates
```

Key decisions:

- **Issuer is separate; validators are embedded.** `eas-weather-rs` and the
  listener validate with the shared signing key (`webserver.jwk_key_file` /
  `SERVER__JWK_KEY_FILE`). No runtime dependency on the auth microservice for
  validation.
- **API key (simple DEV/ops path).** Accept an `X-API-Key` header matching the
  configured key (`webserver.api_key_file`). Self-contained; no issuer needed.
- **Either API key or JWT** is accepted on protected endpoints.
- **Asymmetric keys (RS256 / JWK).** The auth microservice holds the private
  key; consuming services hold only the **public** JWK. Services cannot mint
  tokens, and the config's JWK naming matches this model. (Prefer this over a
  shared symmetric secret.)
- **OpenAPI integration.** Security schemes (`api_key`, `bearer`) are declared
  in the utoipa `ApiDoc` via a `SecurityAddon` (`Modify`), and per-endpoint
  `security(...)` requirements are attached to `#[utoipa::path]` annotations so
  Swagger UI shows an "Authorize" button. utoipa documents but does **not**
  enforce — enforcement is the axum auth middleware.

Implementation phasing:

1. **Now:** OpenAPI security docs + API-key middleware in `eas-weather-rs`.
2. **Now (foundation):** JWT validation middleware (RS256/JWK) so the API is
   ready to accept the auth microservice's tokens with no rework.
3. **Later:** build the **auth microservice** as the issuer (user portal login +
   service tokens), once an actual login/user portal exists.

The auth microservice is deferred because there is no issuer yet; validation can
(and should) be in place first.

## Open questions

- Is the feed delivered as CAP XML or JSON?
- Exactly-once vs at-least-once: assume **at-least-once**, made idempotent by
  dedup.
- Does the listener have reliable local disk to buffer to, or should the durable
  buffer be an external broker (Kafka)?
- Is `identifier` guaranteed globally unique by the feed, or must we dedup on a
  composite key?
- Is there a real token issuer (login service) yet, or is the auth microservice
  deferred until a user portal exists?
