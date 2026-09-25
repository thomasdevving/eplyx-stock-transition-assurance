import { defineConfig } from '@playwright/test';
// The hosted workspace is served by the built `eplyx-cloud` binary over a
// Postgres test database (EPLYX_CLOUD_TEST_DATABASE_URL), seeded by `eplyx sync`.
// Set EPLYX_CHROME to a Chromium executable when Google Chrome is not installed.
const browser = process.env.EPLYX_CHROME ? { launchOptions:{ executablePath:process.env.EPLYX_CHROME } } : { channel:'chrome' };
export default defineConfig({
 testDir:'./frontend/tests/cloud',
 testMatch:'**/*.spec.js',
 use:{ baseURL:'http://127.0.0.1:4390', reducedMotion:'reduce', ...browser },
 webServer:[{ command:'node frontend/tests/cloud/server.mjs 4390', url:'http://127.0.0.1:4390/healthz', reuseExistingServer:false, timeout:60000 }],
 workers:1,
 reporter:'list',
});
