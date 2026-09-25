# Stock Transition frontend

The standalone frontend reuses the visual composition, logo geometry, atmospheric
assets and orbit animation from `thomasdevving/Eplyx` main at
`029811d` (Production Pilot: Real Bundle Hosted CI). The implementation lives in
`frontend/`. All code and assets needed by this application are in this repository;
there are no filesystem dependencies on the reference checkout.

The theme recolors section backgrounds, type accents, orbit lines, stone lighting,
logo materials and studio lighting to blue. The brand places **Stock Transition**
below **Eplyx**. The original two orbit planes keep three stones each; the existing switch now controls application presentation:

- Overview: issuer announcement, observed holdings, transition details.
- Technical: impact mapping, local path probes, readiness gates.

The landing page preserves the hero, method, capabilities and saved result, with a guided analysis form below the hero.
Primary navigation has three links: How it works, Capabilities and Explore evidence.
Detailed path scope, population coverage and provenance live in the evidence viewer;
local reproduction instructions are collapsed by default. Audience, vision and
repeated evidence sections are omitted to reflect the smaller project scope.
Content describes the supported lifecycle project rather than the original
program-upgrade product. No old hosted API, projects console or upgrade-analysis workflow is included. The Phase 15 local API wraps only the existing lifecycle commands.

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
return 404. No deployment has been configured or performed.

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

- **Build.** `node scripts/railway/stage-frontend.mjs <empty-dir>` stages only
  tracked sources and evidence data, plus the three built `.so` programs in
  `artifacts/`. It leaves out Milestone validation records and refuses keypairs,
  `.env` and credential files. Deploy with
  `npx @railway/cli up <dir> --path-as-root --service eplyx-stock`.
  [frontend/Dockerfile](../frontend/Dockerfile) compiles `eplyx-lifecycle`, builds
  `dist/` and runs `node frontend/serve.mjs --production` as the unprivileged
  `node` user. The engine resolves evidence from `/src`, the build-time root.
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

The Phase 15 local API performs fresh offline evaluations through the existing built engine; the browser polls actual job state. This adds no HTTP/RPC capture, notice discovery, signing, transaction submission or new execution replay. The published report remains a separately labeled saved result. See [the Phase 15 report](lifecycle-phase-15-production-report.md) for requests, limits, results and startup.

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
mobile navigation, real WebGL rendering plus its fallback, three active stones,
keyboard interaction, direct report links, exact evidence quantities, original
artifact downloads and missing-asset responses. Screenshots are written to ignored
`test-results/` for visual inspection. They do not query a chain. The Phase 15 browser smoke test additionally runs four real offline engine analyses through the local API.

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
