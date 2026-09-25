import { defineConfig, devices } from '@playwright/test';

const isCi = process.env['CI'] !== undefined;

/** Where `npm start` serves the editor (docs/v1/README.md, "Ports"). */
const EDITOR_URL = 'http://localhost:4260';
/** A cold `npm start` builds the engine before serving: `LP_ENGINE_PREBUILT=1` skips that. */
const EDITOR_START_TIMEOUT_MS = 10 * 60 * 1000;

// End-to-end suites: U6 adds the editor's project (e2e/editor/), H17 the hosted one (e2e/hosted/).
export default defineConfig({
  testDir: 'e2e',
  forbidOnly: isCi,
  retries: isCi ? 1 : 0,
  reporter: isCi ? [['list'], ['html', { open: 'never' }]] : 'list',
  use: { trace: 'retain-on-failure' },
  projects: [
    {
      // U6: draw, animate, export and play in the editor served alone, with no server behind it.
      name: 'editor',
      testDir: 'e2e/editor',
      outputDir: 'e2e/editor/test-results',
      timeout: 120_000,
      expect: { timeout: 10_000 },
      use: {
        ...devices['Desktop Chrome'],
        baseURL: EDITOR_URL,
        locale: 'en-US',
        actionTimeout: 15_000,
      },
    },
  ],
  webServer: [
    {
      command: 'npm start',
      url: EDITOR_URL,
      reuseExistingServer: !isCi,
      timeout: EDITOR_START_TIMEOUT_MS,
    },
  ],
});
