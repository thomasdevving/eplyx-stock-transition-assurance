# Milestone 18: optional cloud sync and team workspace

**Cloud sync is optional. Eplyx execution stays local.**

Every analysis step keeps running on the developer's machine or in CI, with no account, no token and no network beyond the configured Solana RPC:
- `init`, `doctor`, `preflight`, `search`, `reproduce`
- `runs`, `show`, `dashboard`
- the CI gate

The cloud never decides a gate. A synced result is a copy of what the local engine concluded. It is not new evidence, and viewing it never reruns RPC, execution, replay or reproduction.

## Two independent workflows

Local only (unchanged):

```sh
eplyx preflight
eplyx search
eplyx dashboard
```

Optional team sync:

```sh
eplyx login        # browser approval; the CLI never asks for a password
eplyx link         # bind this local project to one cloud project
eplyx sync         # upload complete runs, counterexamples and reproduction records
```

- `eplyx sync <run-id>` syncs one run, and `eplyx sync --latest` syncs the newest complete run.
- `eplyx sync --dry-run` prints what would be sent; add `--json` for the exact documents. A dry run needs no link, token or network.
- `eplyx link --unlink` removes the local link. `eplyx logout` revokes the token on the server and deletes it locally.
- `preflight` never uploads anything automatically. There is no auto-sync setting.

## What is synced

For each **complete** run (one whose `metadata.json` exists), sync uploads these members as exact UTF-8 bytes with their SHA-256:

| Member | Why |
|---|---|
| `metadata.json` | run ID, timestamp, `run_source`, Git commit/branch/dirty, Eplyx version, engine binary hash, candidate and package hashes, gate policy and outcome |
| `result/report.json` | the engine's analytical statuses: conversion, population summary, stress counts, invariant findings, readiness, OfficialTransition, gate reasons, limitations |
| `result/bindings.json` | digests that bind the report to its captures |
| `package/eplyx.json` | public mints, adapter, program ID, terms, invariant definitions, candidate SHA |
| `package/config.json` | public owner, source account, amount and proposed reserve |
| `search/counterexamples.json` | the search result, when `eplyx search` ran |

Sync also uploads:
- the sizes of the other allowlisted artifacts, for display ("stays local · 22 MB");
- every saved counterexample file of a synced run;
- every reproduction record of a synced counterexample.

A reproduction record's free-text `error` has the project root, home directory and any URL replaced (`<project>`, `~`, `<url>`). Every other byte is unchanged, and the local record is never modified.

**Never synced:**
- source code, `eplyx.toml`, `report.md`
- the candidate `.so` or any program bytes
- the population, stress, wave, conversion and wallet captures, including the RPC provider origin
- environment variables, RPC URLs or credentials, Solana keys, local absolute paths
- the Eplyx token

Public on-chain addresses in the files above are included. The `--artifacts` upload mode is deferred, so this milestone ships metadata sync only.

**Refusal instead of rewriting.** Synced artifacts are exact engine bytes, so they are never edited. The CLI checks every artifact text before upload, and the server checks it again on receipt. Sync is refused if the text contains any of:
- a URL of any scheme (`://`);
- an absolute path (`/Users/`, `/home/`, `/private/`, `/tmp/`, drive letters and similar);
- a credential marker;
- the exact value of the local `SOLANA_RPC_URL`, `EPLYX_TOKEN` or stored token (client-side check).

## Accounts, tokens and storage

- **Browser.** Sign-up and sign-in use email and password in the hosted web app only. Passwords are stored as Argon2id hashes. Sessions use an `HttpOnly; SameSite=Strict` cookie (`Secure` over HTTPS). Every cookie-authenticated write, and sign-up and sign-in, must come from the site's own origin. Failed sign-ins are throttled per email. `EPLYX_CLOUD_SIGNUP_CODE` optionally restricts sign-up.
- **CLI.** `eplyx login` uses a device-authorization flow:
  1. The CLI shows a code and a URL on the server.
  2. A signed-in browser session confirms the code. A CLI token cannot approve codes.
  3. The CLI polls for the token.

  The CLI receives a user token (`eplyx_u_…`, 90 days). Device codes expire after 10 minutes and are single use.
- **CI.** A workspace owner creates a project-scoped CI token (`eplyx_ci_…`) on the project's Settings page. It is shown once. A CI token can only:
  - sync runs, counterexamples and reproduction records to its own project;
  - read that project's identity (for `link`).

  It cannot read dashboards, list workspaces or manage anything. Revocation takes effect immediately.
- **Server storage.** The server stores only the SHA-256 of every session and token.
- **Local storage.** `credentials.json` holds one token per server origin, with no Solana, RPC or GitHub credential. It lives in `EPLYX_CONFIG_DIR`, `%APPDATA%\eplyx` (Windows), `$XDG_CONFIG_HOME/eplyx` or `~/.config/eplyx`.
  - Unix: the file is `0600` in a `0700` directory. A group- or world-readable file is refused.
  - Windows: it relies on the per-user profile ACL.
  - The OS keychain is not used; it would add platform dependencies to the release builds.
- **Environment.** `EPLYX_TOKEN`, `EPLYX_PROJECT_ID` and `EPLYX_CLOUD_URL` override the stored values. HTTPS is required except on loopback, and the client refuses redirects, so a token never follows to another host.
- **Token boundary.** Only `login`, `logout`, `link` and `sync` read the token. Every other command removes `EPLYX_TOKEN` from its own environment before doing anything else. The offline VM worker (which replay also uses) starts with an empty environment. No canonical artifact, report, counterexample or reproduction record ever contains the token. Tests cover all four claims.

## Workspace and project model

```
User → Workspace (owner | member) → Project (private to the workspace)
     → Runs → Counterexamples → Reproduction records
```

- **Sign-up.** Creates a personal workspace with the new user as owner.
- **Owners.** Add existing accounts by email, remove members, and create or revoke CI tokens.
- **Members.** View every project in the workspace, create projects and sync.
- **Visibility.** There is no public visibility. A public repository never makes a project public.
- **Link record.** `.eplyx/project.json` keeps its stable local `id` and gains an additive `cloud` object: `{server, workspace_id, project_id, linked_at}`. That links the local project to at most one cloud project; `--force` relinks it.
- **Local and cloud IDs.** The local project ID is derived from the checkout path, so each developer checkout and each CI workspace has its own. They stay distinct from the cloud `prj_` ID.
- **Server links.** The server records which local project IDs feed a cloud project in `project_links`. CI links implicitly on its first sync.
- **Browsers cannot link.** A browser session cannot create a link (`403`), and the local dashboard remains GET-only.

## Sync protocol, idempotency and conflicts

A run's cloud key is (cloud project, local run ID). Its **core digest** is the SHA-256 of the canonical JSON of:
- the document schema, local project ID and run ID;
- the digests of metadata, report, bindings, manifest and config.

On receipt the server re-runs the engine checks:
1. Digests match the exact bytes, and the privacy checks pass.
2. The metadata parses, and `run_id` matches.
3. `transition_package_sha256` is **recomputed** from the manifest and config through the engine's package identity function. This uses the program SHA the manifest declares; program bytes never travel.
4. The report, bindings and metadata agree on package, config, gate policy and outcome.
5. OfficialTransition is `NotTested`, and no funds moved.
6. The search belongs to this package run.
7. The engine view model parses the run as Complete with no problems.

Only then does it store the document.

| Situation | Answer |
|---|---|
| new run | `201 created` |
| same core digest (and same or no search) | `200 unchanged`: retry-safe |
| same core digest, search absent before and present now | `200 search_attached`, exactly once |
| same run ID, different core digest (including another local project) | `409 conflict`; nothing is overwritten |
| same run, a different search digest | `409 conflict` |

Postgres triggers make synced runs, counterexamples and reproduction records immutable at the database level, apart from that one append-only search attachment. Workspaces and projects can have cloud-only presentation metadata (names). Analytical content cannot change.

**Counterexamples.** A counterexample is accepted only if:
- its file parses;
- the engine recomputes the same content-addressed `cx_` ID;
- its parent run is already synced to the **same project** from the **same local project**;
- that run's synced search has exactly the recorded `search_sha256` and contains this exact engine counterexample.

**Reproduction records.** A record is accepted only if its counterexample is synced to the same project from the same local project, and any recorded parent run or search digest matches.

**Syncing is safe to interrupt.** Each document is its own idempotent request, run documents commit in one transaction, and the CLI posts runs, then counterexamples, then reproductions. An unreachable server stops the sync with:

> could not reach Eplyx cloud … Nothing local changed … retry

The command exits `2`, and the next `eplyx sync` resumes.

**The sync sidecar.** Each run's last attempt is recorded in `.eplyx/sync/runs/<run>.json`: status, cloud project, digests, counts, times, error and URL. This is operational metadata. Canonical run artifacts are never touched. A failed sync never changes a preflight, search or gate result or exit code.

## Local and CI history in one project

- A developer runs `eplyx link` once, then `eplyx sync`.
- CI sets `EPLYX_TOKEN` (a CI token) and `EPLYX_PROJECT_ID`, and runs `eplyx preflight` followed by `eplyx sync --latest`. It needs no link or credentials file. See [examples/ci/eplyx-cloud-sync.yml](../examples/ci/eplyx-cloud-sync.yml):
  - fork pull requests never receive secrets;
  - the sync step is skipped without a token;
  - `continue-on-error` means a cloud outage cannot change the gate;
  - the preflight exit code is re-raised.
- `run_source` comes only from the engine's metadata (`local`, or `ci` when `CI` was set). Schema 1 runs show **Not recorded**, and the source is never inferred. Who synced a run, when, and through which token type is recorded separately as provenance.
- Runs are numbered by run ID across all sources, the same rule the local dashboard uses.

## Hosted dashboard

The hosted pages reuse the local dashboard modules (`frontend/dashboard/`). A small `env.js` points them at a project path (`/p/<prj>`) and at the hosted view API. The same pages are available:
- Overview, Runs, Run detail, Counterexamples, Compare
- Production State, Invariants, CI / Gate, Project

The cloud adds:
- a workspace/project switcher, sign-out and a Settings page for CI tokens;
- a permanent **Synced results** banner, and a provenance line on each run: "Synced result · Local run · date · commit · synced by … via …";
- "Release health": the latest local and latest CI result side by side, with counterexample and reproduction counts. There is no combined score.

Artifacts show "stays local", and there are no download links for captures or program bytes. Counterexample pages show:
- observed vs derived, the parent run, dimension, account, boundary, failure signature, rollback and minimization;
- the synced reproduction history, including failed attempts;
- a copyable `eplyx reproduce <cx-id>`. The cloud never runs it.

**No analytical computation in the browser.** The server builds every payload with the engine's own `dashboard::view` functions over the synced bytes:
- `summary`, `assemble`, `run_detail_for`, `counterexample_detail_for`, `project_payload`, the engine gate evaluator;
- `compare_runs`, which carries the exact Milestone 16 search-equivalence and counterexample-status semantics.

A test asserts that the hosted comparison equals the local dashboard's comparison field for field. The **"Search domains differ — counterexample disappearance does not prove resolution."** banner appears unchanged.

**Local dashboard.** It now shows "Eplyx cloud: linked to prj_… · n of m runs synced" or "not linked", plus a per-run `synced` or `sync failed` tag. It reads the link and sidecar only. It needs no login and still cannot change anything.

**Hero page.** The main frontend's hero, header and footer link to the hosted workspace (`frontend/src/workspace.js`, `CLOUD_WORKSPACE_URL`). Following the link sends nothing; runs arrive only through `eplyx sync`.

**Public demo.** At most one project can be published read-only, at `/demo`, and only by the server operator through `EPLYX_CLOUD_DEMO_PROJECT`. It serves the same sanitized synced metadata and hides who linked what. There is no public discovery, and no way for a project owner to make a project public.

## API

All JSON, with strict schemas (`deny_unknown_fields`). Default body bound 64 KiB; sync bounds are run 12 MiB, counterexample 3 MiB and reproduction 64 KiB. Oversized requests get `413`, invalid documents `422`. Unknown and inaccessible projects both answer `404`.

```
GET    /healthz
POST   /api/v1/auth/signup | login | logout
POST   /api/v1/auth/device            GET /api/v1/auth/device/lookup?code=
POST   /api/v1/auth/device/approve    POST /api/v1/auth/device/token
DELETE /api/v1/auth/token             GET  /api/v1/me
GET|POST /api/v1/workspaces           GET|POST /api/v1/workspaces/:ws/members
DELETE /api/v1/workspaces/:ws/members/:user
POST   /api/v1/workspaces/:ws/projects
GET    /api/v1/projects/:id           POST /api/v1/projects/:id/links
GET|POST /api/v1/projects/:id/ci-tokens   DELETE /api/v1/projects/:id/ci-tokens/:token
POST   /api/v1/projects/:id/runs | counterexamples | reproductions
GET    /api/v1/projects/:id/runs | counterexamples
GET    /api/v1/projects/:id/view/{project,runs,runs/:run,counterexamples,counterexamples/:cx,counterexamples/:cx/raw,compare}
GET    /api/v1/demo/view/…            (only when EPLYX_CLOUD_DEMO_PROJECT is set)
```

There is no arbitrary query or file API.

## Hosting (Railway)

**Components:**
- one `eplyx-cloud` service, built from [cloud/Dockerfile](../cloud/Dockerfile) through [railway.json](../railway.json) with health check `/healthz`;
- one Railway Postgres database.

There are no workers, no object storage, no RPC and no VM.

| Variable | Required | Meaning |
|---|---|---|
| `DATABASE_URL` | yes | Railway Postgres **private** URL (`${{Postgres.DATABASE_URL}}`); no TLS layer is added on the private network |
| `PORT` | set by Railway | listen port |
| `EPLYX_CLOUD_PUBLIC_URL` | recommended | public https origin; defaults to `https://$RAILWAY_PUBLIC_DOMAIN` |
| `EPLYX_CLOUD_SIGNUP_CODE` | optional | require this code to create accounts |
| `EPLYX_CLOUD_DEMO_PROJECT` | optional | the single `prj_…` published read-only at `/demo` |

The service holds no Solana RPC credential; it never captures or executes. Migrations are embedded SQL (`cloud/migrations/`), applied at startup in order under a Postgres advisory lock and recorded in `schema_migrations`. `GET /healthz` returns `{"ok":true,"database":"reachable","executes":false}`, or `503` when the database is unreachable.

## Tests

- **`make test`.** Runs the engine contract tests (`engine/tests/cloud_contract.rs`):
  - fixture documents verify; engine summaries from synced bytes equal the local summaries;
  - tampered digests, URLs, paths, package or config edits, renamed runs, gate mismatches, unknown fields and oversize members are refused;
  - candidate bytes and captures contribute sizes only;
  - counterexample and reproduction bindings hold, and reproduction errors are sanitized;
  - the local dashboard shows link and sync state read-only;
  - the CLI refuses a sync without a link or login; a dry run leaks no RPC URL or token; an offline sync is retryable and changes nothing; local commands work with a token present.
- **Unit tests.**
  - the privacy scanner;
  - server-origin rules;
  - owner-only credentials;
  - link round trip;
  - the offline VM worker inheriting no `EPLYX_TOKEN` or `SOLANA_RPC_URL`;
  - passwords and codes.
- **`make test-cloud`.** Needs `EPLYX_CLOUD_TEST_DATABASE_URL` and fails without it. It runs the API tests against real Postgres:
  - unauthenticated and forged access is rejected; workspace isolation holds; projects are private by default;
  - schema, digest, privacy and size rejection;
  - idempotency, search attachment, conflicts and DB-level immutability;
  - counterexample and reproduction parent binding;
  - CI-token scope and revocation; same-origin sessions and no browser relink;
  - device-flow single use;
  - hosted views equal the local engine views;
  - the demo project is read-only.

  It also runs the CLI tests with the real `eplyx` binary: login, link, sync, retry, logout, CI environment sync, conflict and revoked-token refusal.
- **`npm run test:cloud`.** Hosted browser pages over a seeded server.

## Deferred

Deferred to keep this milestone safe:
- `--artifacts` object storage
- OS keychain storage
- labels and comments
- email, Slack and billing
- enterprise RBAC and GitHub OAuth automation
- remote replay or execution
- public project discovery
