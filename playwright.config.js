import { defineConfig } from '@playwright/test';
const port=process.env.EPLYX_TEST_PORT || '4183';
export default defineConfig({
 testDir: './frontend/tests',
 testMatch: '**/*.spec.js',
 testIgnore: '**/dashboard/**',
 use: { baseURL: `http://127.0.0.1:${port}`, channel: 'chrome', reducedMotion: 'reduce' },
 webServer: { command: 'npm run dev', url: `http://127.0.0.1:${port}`, env:{PORT:port}, reuseExistingServer:false },
 reporter: 'list',
});
