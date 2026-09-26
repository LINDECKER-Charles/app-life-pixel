import { existsSync } from 'node:fs';
import { rm } from 'node:fs/promises';
import { dirname, join } from 'node:path';
import { ADMIN_STORAGE_STATE, createAdmin, shareAdmin, signInAdmin } from './stack/admin';
import { buildBinaries, REPOSITORY_ROOT } from './stack/binaries';
import { HostedStack } from './stack/hosted-stack';
import { expectFree } from './stack/network';
import { PORTS } from './stack/ports';

/** What `npm run build` leaves, and the server serves. */
const BUILT_APP = join(REPOSITORY_ROOT, 'frontend/dist/app/browser/index.html');

/**
 * The project `hosted`'s global setup (support-admin.md, H17), once `npm run build` has run: the
 * server and the admin server built with `stack-tests`, each on a test database of its own from
 * its `test-database create`, started on ports 8460 to 8464 with a 200,000-byte quota, and an
 * admin made with `create-admin --password-stdin` and signed in. The function it returns is the
 * teardown: it stops both binaries and drops both databases.
 */
export default async function globalSetup(): Promise<() => Promise<void>> {
  const envFile = join(REPOSITORY_ROOT, '.env');
  if (existsSync(envFile)) process.loadEnvFile(envFile);
  if (!existsSync(BUILT_APP)) throw new Error(`${BUILT_APP} is missing: run npm run build first.`);
  await expectFree(Object.values(PORTS));
  await buildBinaries();
  const stack = new HostedStack();
  const teardown = async (): Promise<void> => {
    await stack.stop();
    await rm(dirname(ADMIN_STORAGE_STATE), { recursive: true, force: true });
  };
  try {
    await stack.start();
    const admin = await createAdmin(stack.adminEnv());
    shareAdmin({ ...admin, usedStep: await signInAdmin(admin) });
  } catch (error) {
    await teardown();
    throw error;
  }
  return teardown;
}
