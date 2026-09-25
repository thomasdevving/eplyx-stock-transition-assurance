# Stock Transition frontend

The standalone frontend reuses the visual composition, logo geometry, atmospheric
assets and orbit animation from `thomasdevving/Eplyx` main at
`029811d` (Production Pilot: Real Bundle Hosted CI). The implementation lives in
`frontend/`. All code and assets needed by this application are in this repository;
there are no filesystem dependencies on the reference checkout.

The theme recolors section backgrounds, type accents, orbit lines, stone lighting,
logo materials and studio lighting to blue. The brand places **Stock Transition**
below **Eplyx**. The original two orbit planes keep three stones each; the switch
controls application presentation:

- Overview: real production state, your program and an exact answer, in plain language.
- Technical: the preflight CLI, browser VM checks, and evidence with optional team history.

The landing page tells one story, in this order:
1. **Hero:** Eplyx rehearses a transition against real Solana accounts before it goes live.
2. **How it works:** five steps, in plain words (Overview) and exact terms (Technical).
3. **What you get:** six capabilities, plus the dashboard and team workspace.
4. **A real run:** the same demo program with a funded and an empty reserve, PASS WITH WARNINGS against BLOCKED. Every figure comes from `frontend/src/showcase.js` and is checked against the saved runs in `fixtures/dashboard/` by `npm run check:frontend`.
5. **Holder impact:** the saved SPACEX example.
6. **Trust:** what Eplyx does and does not prove.
7. **Getting started:** install and first commands.

The analysis page opens with a three-step explanation above the form.
Primary navigation links to How it works, Capabilities, Example run, Team workspace and Explore evidence.
Detailed path scope, population coverage and provenance live in the evidence viewer;
local reproduction instructions are collapsed by default. Audience, vision and
repeated evidence sections are omitted to reflect the smaller project scope.
Content describes the lifecycle project rather than the original program-upgrade
product. The legacy hosted upgrade API and projects console are not included. The
Stock Transition Node API serves fresh bounded reads, selected current-wallet
checks, proposed-transition evaluations and candidate-conversion stress runs;
these are separate from the `eplyx` operator CLI and its package workflow.

## Run and build

Use Node 22.6 or later:

```sh
npm ci
npm run build:engine
npm run dev
```

The development server listens on `127.0.0.1:4173`. Set `PORT` to choose another
port. `npm run build` creates a static `dist/`, and `npm start` serves that build.
A static host must serve `index.html` for `/` and `/evidence` (including its trailing
slash variant), and serve `/src/` and `/public/` as files. Unknown assets must
return 404. This describes the static build only; the hosted demo uses the
separate Node service and Docker deployment described below.

Three.js is pinned in the lockfile and copied locally at build time. The original
Google Fonts stylesheet supplies Manrope and DM Sans, with system-font fallbacks.
The 3D mark falls back to the original vector silhouette if WebGL is unavailable.
Reduced-motion preferences disable orbital motion and reveal animations. Mobile
navigation, keyboard focus, expandable path details and direct section links are
supported.

## Hosted deployment (Railway)

The frontend and its analysis service can also run as a hosted demo:
**https://eplyx-stock-production.up.railway.app**. Locally nothing changes: the
server still listens on `127.0.0.1` with no access code.

- **Build.** [frontend/Dockerfile](../frontend/Dockerfile) builds entirely from
  tracked sources in three stages:
  1. It compiles `eplyx-lifecycle`.
  2. It builds the registered demo conversion program with the pinned
     `cargo-build-sbf` 4.4.0, the same tool the release workflow uses.
  3. It builds `dist/` and runs `node frontend/serve.mjs --production` as the
     unprivileged `node` user.

  The engine resolves evidence from `/src`, the build-time root. The shared root
  `.dockerignore` keeps local `artifacts/`, keypairs, run stores, build output and
  Milestone validation records out of the context.
- **Deploy from GitHub.** Connect the `eplyx-stock` service to this repository's
  `main` branch and set its config file to `frontend/railway.json`. That file sets
  the Dockerfile, the watch paths and the `/` health check, so only relevant pushes
  rebuild.
- **Manual deploy.** `node scripts/railway/stage-frontend.mjs <empty-dir>` stages
  exactly the tracked files. Then run
  `npx @railway/cli up <dir> --path-as-root --service eplyx-stock`.
- **Access code.** With `EPLYX_ANALYSIS_ACCESS_CODE` set, every analysis API
  request needs an unlocked browser session. That covers starting, polling and
  reading runs, checks, preflights, conversions and stress tests. Health,
  catalogue, stress budget and mechanism information stay public.
  - The browser asks for the code once, in a dialog, and then retries.
  - The unlock is an HttpOnly, SameSite=Strict, Secure cookie holding an HMAC of
    the session. It is bound to both the session and the code, so changing the
    code locks every browser out again.
  - Wrong codes are limited to 10 per client per 15 minutes.
  - The code only decides who may ask for an analysis. It never changes a
    result.
- **Variables.**

  | Variable | Meaning |
  |---|---|
  | `SOLANA_RPC_URL` | read-only mainnet RPC for fresh analyses; only fresh engine children receive it, and results record scheme and host only |
  | `EPLYX_ANALYSIS_ACCESS_CODE` | the access code; share it only with people who may spend RPC quota |
  | `EPLYX_PUBLIC_ORIGIN` | public https origin; the same-origin check uses it |
  | `RAILWAY_DOCKERFILE_PATH` | `frontend/Dockerfile` |
  | `HOST`, `PORT`, `EPLYX_ENGINE`, `EPLYX_RUN_DIRECTORY` | set by the Dockerfile |

  Without `SOLANA_RPC_URL`, the saved example and published evidence still
  work, and fresh inspections report that the provider is unavailable.
- **Storage.** Analysis runs live in `/data/analysis-runs` inside the container
  and are lost on redeploy. They are session results, not evidence.

## Evidence boundary

`frontend/evidence.mjs` reads five published reports and requires exact pinned
SHA-256 hashes before copying their original bytes into generated public assets:

- Phase 12 notice readiness.
- Phase 9 direct-holder path resolution.
- Phase 10 native LP withdrawal.
- Phase 12 normalized lifecycle event.
- Phase 14 candidate rollout assessments and original local stub observations.

It generates a small browser summary from these files; counts, findings, exact
path attempts, principal/fee quantities and identities are not invented demo data.
Raw amounts are formatted with `BigInt`, preserving integer precision. The viewer
provides the original JSON downloads and production-report documentation.
Generated assets, vendor files and `dist/` are ignored by Git and rebuilt locally.

This is a display of existing published reports, not a new validation of the full
engine evidence bundle. Build-time report pins do not regenerate notices, replay
transactions, re-evaluate readiness or attest to live state. Engine commands retain
those responsibilities. Updating published evidence requires review of both the
pins and explanatory landing-page copy; a changed pinned report fails the build.

The UI preserves these distinctions:

- Notice assertions, captured mint identities and explicit demo time are separate.
- OfficialTransition remains NotTested; conversion mechanics and ratio are unknown.
- Direct transfer and secondary-market exit are separate conditional proofs.
- One LP principal withdrawal leaves accrued fees and the position account.
- Local signer privilege is assumed; key possession is unknown.
- Entity/amount/path/venue/range/fraction/bank/scenario scope stays attached to proof.
- Four fully represented mobility samples do not prove the other 10,151 positive
  entities. Independent amounts/routes are not summed into simultaneous capacity.
- Incomplete is the finding under the demo assurance policy, not an asset verdict.

## On-demand analysis and execution boundary

The `/analysis` page calls the same-origin Node job API. The server uses its
configured read-only mainnet RPC provider for bounded current token and public
owner captures. It can inspect a catalogue asset or validated custom mint; a
token overview can request at most 20 largest accounts, while wallet scope queries
the supplied owner + mint. These scopes do not discover protocol positions or
establish a complete holder population.

For one account discovered in that exact wallet run, the browser can request a
freshly captured Transfer or supported Meteora DLMM market-exit check in the
analysis service’s VM. It can also evaluate a prospective user-proposed lifecycle
time and test bounded conversion terms with the registered **Eplyx Demo Candidate
Conversion** program. Only after that conversion passes does the browser offer a
stress test of the same plan against a frozen bounded sample of current accounts.
Fresh state uses the configured read-only RPC; LiteSVM receives no RPC credential,
and no network transaction is constructed or submitted. Local signer privilege
is assumed; possession of the key remains unknown.

The browser candidate-conversion plan does not execute uploaded or operator-built
program bytes, and it is not an issuer conversion. `OfficialTransition` remains
NotTested. The LP-principal withdrawal shown in `/evidence` is pinned earlier
evidence; the browser does not expose a fresh withdrawal action. Every current
check starts untested in a new run and remains bound to its exact account, amount,
route or terms, capture and program identity. See the
[on-demand milestone record](on-demand-progress.md) for the full history and
[developer CLI quick start](developer-cli.md) for package, gate, replay, dashboard
and cloud commands. Milestones 8–14 add the operator package, stress, search and
gate workflow; Milestone 15 adds the presentation switch and job API; Milestones
16–18 add the CLI dashboard, verified releases and optional cloud sync. The
browser does not expose the package workflow, deployment gate, replay/search
commands or cloud sync.

## Validation

```sh
npm run check:frontend
npm run build
npm run test:frontend
npm run test:dashboard   # local dashboard; set EPLYX_CHROME if Chrome is not installed
make test
make fmt-check
make lint
```

Browser tests use Playwright with installed Google Chrome. They check desktop and
mobile navigation, real WebGL rendering plus its fallback, the three stones in
each view, keyboard interaction, exact evidence quantities, artifact downloads,
API boundaries and analysis flows. Deterministic tests use fixtures; explicitly
opt-in live cases can query the configured RPC. Screenshots are written to ignored
`test-results/` for visual inspection.

## Phase 14 published case panel

The evidence viewer adds one case panel using the existing expandable path details and
scope-row styles. Its four submitted plans show claim assessments, explicitly identified
original/derived policies, readiness scope/status and actual retained local marker
observations as separate fields. All results come from the pinned engine artifact;
the frontend does not parse claims, compute assurance or run the stub. The original
population verdict remains Incomplete. Narrow principal-removal Ready cannot be
presented as complete exit, conversion, population assurance or future availability.
The published marker is an inert local demonstration observation, not a real issuer
intervention. Existing composition, assets, navigation and engine proof displays
remain in place.
