import { defineConfig } from '@playwright/test';

const isCi = process.env['CI'] !== undefined;

// End-to-end suites: U6 adds the editor's project (e2e/editor/), H17 the hosted one (e2e/hosted/).
export default defineConfig({
  testDir: 'e2e',
  forbidOnly: isCi,
  retries: isCi ? 1 : 0,
  reporter: isCi ? [['list'], ['html', { open: 'never' }]] : 'list',
  use: { trace: 'retain-on-failure' },
  projects: [],
});
