import { defineConfig, devices } from '@playwright/test';

const isCi = process.env['CI'] !== undefined;

// The loader's tests: every file is served through `page.route`, so no server runs.
export default defineConfig({
  testDir: 'tests',
  forbidOnly: isCi,
  retries: isCi ? 1 : 0,
  reporter: 'list',
  use: { trace: 'retain-on-failure' },
  projects: [{ name: 'chromium', use: { ...devices['Desktop Chrome'] } }],
});
