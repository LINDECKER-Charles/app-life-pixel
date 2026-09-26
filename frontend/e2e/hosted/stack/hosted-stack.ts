import type { ChildProcess } from 'node:child_process';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { REPOSITORY_ROOT, runBinary, startBinary, stopBinary, type Binary } from './binaries';
import { waitHealthy } from './network';
import { ADMIN_CONSOLE_URL, HOSTED_APP_URL, PORTS } from './ports';

/** The free plan's storage in this suite: small enough for a journey to fill it. */
export const STORAGE_QUOTA_BYTES = 200_000;

const APP_DIR = join(REPOSITORY_ROOT, 'frontend/dist/app/browser');
const CONSOLE_DIR = join(REPOSITORY_ROOT, 'frontend/dist/admin/browser');
const I18N_DIR = join(REPOSITORY_ROOT, 'i18n');

/**
 * The server and the admin server as the journeys use them: each on a test database of its own,
 * made by its `test-database create`, the documents in a temporary folder, the free plan's
 * storage at 200,000 bytes. The rest of the configuration is `.env`'s, which both read from the
 * repository's root. `stop` undoes whatever `start` did, even halfway.
 */
export class HostedStack {
  private storageDir?: string;
  private serverDatabase?: string;
  private adminDatabase?: string;
  private readonly processes: ChildProcess[] = [];

  /** The admin server's test database, where `create-admin` writes. */
  get adminDatabaseUrl(): string {
    if (!this.adminDatabase) throw new Error('The stack is not started.');
    return this.adminDatabase;
  }

  async start(): Promise<void> {
    // The documents stay out of S3Mock's shared bucket: the server's sweeper, which runs as it
    // starts, deletes old documents that its own database does not reference.
    this.storageDir = await mkdtemp(join(tmpdir(), 'life-pixel-e2e-hosted-'));
    this.serverDatabase = await createTestDatabase('life-pixel-server');
    this.adminDatabase = await createTestDatabase('life-pixel-admin-server');
    await this.serve('life-pixel-server', this.serverEnv(), PORTS.app);
    await this.serve('life-pixel-admin-server', this.adminEnv(), PORTS.console);
  }

  async stop(): Promise<void> {
    await Promise.all(this.processes.splice(0).map(stopBinary));
    if (this.serverDatabase) await dropTestDatabase('life-pixel-server', this.serverDatabase);
    if (this.adminDatabase) await dropTestDatabase('life-pixel-admin-server', this.adminDatabase);
    this.serverDatabase = this.adminDatabase = undefined;
    if (this.storageDir) await rm(this.storageDir, { recursive: true, force: true });
  }

  /** The environment of the admin server's subcommands, such as `create-admin`. */
  adminEnv(): NodeJS.ProcessEnv {
    return {
      ...process.env,
      LPA_HTTP_ADDR: `127.0.0.1:${PORTS.console}`,
      LPA_METRICS_ADDR: `127.0.0.1:${PORTS.consoleMetrics}`,
      LPA_PUBLIC_URL: ADMIN_CONSOLE_URL,
      LPA_APP_DIR: CONSOLE_DIR,
      LPA_I18N_DIR: I18N_DIR,
      LPA_DATABASE_URL: this.adminDatabaseUrl,
      LPA_SERVER_ADMIN_API_URL: `http://127.0.0.1:${PORTS.adminApi}/internal/admin/v1`,
    };
  }

  private serverEnv(): NodeJS.ProcessEnv {
    return {
      ...process.env,
      LP_HTTP_ADDR: `127.0.0.1:${PORTS.app}`,
      LP_METRICS_ADDR: `127.0.0.1:${PORTS.metrics}`,
      LP_ADMIN_API_ADDR: `127.0.0.1:${PORTS.adminApi}`,
      LP_PUBLIC_URL: HOSTED_APP_URL,
      LP_APP_DIR: APP_DIR,
      LP_I18N_DIR: I18N_DIR,
      LP_DATABASE_URL: this.serverDatabase,
      LP_STORAGE_URL: `file://${this.storageDir}`,
      LP_PLAN_FREE_STORAGE_BYTES: String(STORAGE_QUOTA_BYTES),
      // Each journey sends X-Forwarded-For with an address of its own, so that the per-address
      // rate limits — 5 sign-ups an hour — count journeys apart, as they would people.
      LP_TRUSTED_PROXIES: '127.0.0.1,::1',
    };
  }

  private async serve(binary: Binary, env: NodeJS.ProcessEnv, port: number): Promise<void> {
    const child = await startBinary(binary, env);
    this.processes.push(child);
    await waitHealthy(child, port, binary);
  }
}

async function createTestDatabase(binary: Binary): Promise<string> {
  const url = (await runBinary(binary, ['test-database', 'create'])).trim();
  if (!/^postgres(ql)?:\/\//.test(url)) throw new Error(`${binary} printed no database URL.`);
  return url;
}

async function dropTestDatabase(binary: Binary, url: string): Promise<void> {
  await runBinary(binary, ['test-database', 'drop', url]).catch((error: unknown) => {
    console.error(`Could not drop ${binary}'s test database:`, error);
  });
}
