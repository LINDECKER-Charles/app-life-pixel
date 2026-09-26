import { defineConfig, devices } from '@playwright/test';
import { HOSTED_APP_URL } from './e2e/hosted/stack/ports';

const isCi = process.env['CI'] !== undefined;

/** Where `npm start` serves the editor (docs/v1/README.md, "Ports"). */
const EDITOR_URL = 'http://localhost:4260';
/** A cold `npm start` builds the engine before serving: `LP_ENGINE_PREBUILT=1` skips that. */
const EDITOR_START_TIMEOUT_MS = 10 * 60 * 1000;

/**
 * Whether this run includes the project `name`: every project runs when none is named. The
 * editor's dev server and the hosted stack are global to a run, so each starts only when its
 * project runs.
 */
function runs(name: string): boolean {
  const named = process.argv.flatMap((argument, index, all) => {
    if (argument.startsWith('--project=')) return [argument.slice('--project='.length)];
    return argument === '--project' ? [all[index + 1] ?? ''] : [];
  });
  return named.length === 0 || named.includes(name);
}

// End-to-end suites: U6 adds the editor's project (e2e/editor/), H17 the hosted one (e2e/hosted/).
export default defineConfig({
  testDir: 'e2e',
  forbidOnly: isCi,
  retries: isCi ? 1 : 0,
  reporter: isCi ? [['list'], ['html', { open: 'never' }]] : 'list',
  use: { trace: 'retain-on-failure' },
  // H17: the server and the admin server, on test databases of their own; see e2e/hosted/.
  globalSetup: runs('hosted') ? './e2e/hosted/global-setup.ts' : undefined,
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
    {
      // H17: M3's journeys on the built app, served by the server on the local stack, and the
      // admin console served by the admin server.
      name: 'hosted',
      testDir: 'e2e/hosted',
      outputDir: 'e2e/hosted/test-results',
      timeout: 180_000,
      expect: { timeout: 15_000 },
      workers: 4,
      use: {
        ...devices['Desktop Chrome'],
        baseURL: HOSTED_APP_URL,
        locale: 'en-US',
        actionTimeout: 15_000,
      },
    },
  ],
  webServer: runs('editor')
    ? [
        {
          command: 'npm start',
          url: EDITOR_URL,
          reuseExistingServer: !isCi,
          timeout: EDITOR_START_TIMEOUT_MS,
        },
      ]
    : [],
});
