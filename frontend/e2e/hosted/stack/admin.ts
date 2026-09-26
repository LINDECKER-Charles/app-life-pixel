import { mkdir } from 'node:fs/promises';
import { dirname, resolve } from 'node:path';
import { request } from '@playwright/test';
import { randomEmail, randomPassword } from '../identities';
import { authenticator, freshCode } from '../totp';
import { runBinary } from './binaries';
import { ADMIN_CONSOLE_URL } from './ports';

/** The variable carrying the admin from the global setup to the journeys, as JSON. */
const ADMIN_VARIABLE = 'LP_E2E_ADMIN';
/** The admin's signed-in browser state: its session cookie. Ignored by Git. */
export const ADMIN_STORAGE_STATE = resolve(__dirname, '../.auth/admin.json');

/** The admin the global setup creates and signs in. */
export interface Admin {
  readonly email: string;
  readonly password: string;
  /** The `otpauth://` URI `create-admin` printed. */
  readonly otpauthUri: string;
  /** The TOTP step the global setup signed in with: the next sign-in needs a later one. */
  readonly usedStep: number;
}

/** Creates an admin with `create-admin --password-stdin`, and reads its URI from the output. */
export async function createAdmin(env: NodeJS.ProcessEnv): Promise<Omit<Admin, 'usedStep'>> {
  const email = randomEmail('admin');
  const password = randomPassword();
  const output = await runBinary(
    'life-pixel-admin-server',
    ['create-admin', email, '--password-stdin'],
    {
      env,
      input: `${password}\n`,
    },
  );
  const otpauthUri = output.split('\n').find((line) => line.startsWith('otpauth://totp/'));
  if (!otpauthUri) throw new Error('create-admin printed no otpauth:// URI.');
  return { email, password, otpauthUri };
}

/**
 * Signs the admin in through the console's API and keeps the session in `ADMIN_STORAGE_STATE`,
 * so that the journeys open the console signed in; returns the step its code used.
 */
export async function signInAdmin(admin: Omit<Admin, 'usedStep'>): Promise<number> {
  const { code, step } = await freshCode(authenticator(admin.otpauthUri));
  const context = await request.newContext({
    baseURL: ADMIN_CONSOLE_URL,
    extraHTTPHeaders: { Origin: ADMIN_CONSOLE_URL },
  });
  try {
    const data = { email: admin.email, password: admin.password, code };
    const response = await context.post('/api/admin/v1/auth/sign-in', { data });
    if (!response.ok()) {
      throw new Error(`The admin sign-in answered ${response.status()}: ${await response.text()}`);
    }
    await mkdir(dirname(ADMIN_STORAGE_STATE), { recursive: true });
    await context.storageState({ path: ADMIN_STORAGE_STATE });
    return step;
  } finally {
    await context.dispose();
  }
}

/** Hands the admin to the journeys, which run in other processes. */
export function shareAdmin(admin: Admin): void {
  process.env[ADMIN_VARIABLE] = JSON.stringify(admin);
}

/** The admin the global setup created, in a journey. */
export function sharedAdmin(): Admin {
  const value = process.env[ADMIN_VARIABLE];
  if (!value) throw new Error(`${ADMIN_VARIABLE} is unset: run the project "hosted".`);
  return JSON.parse(value) as Admin;
}
