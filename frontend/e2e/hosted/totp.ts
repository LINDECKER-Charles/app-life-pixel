import { setTimeout as sleep } from 'node:timers/promises';
import { TOTP, URI } from 'otpauth';

/** A code and the time step it belongs to. */
export interface TotpCode {
  readonly code: string;
  readonly step: number;
}

/** The authenticator of an `otpauth://totp/…` URI, as `create-admin` prints it. */
export function authenticator(uri: string): TOTP {
  const otp = URI.parse(uri);
  if (!(otp instanceof TOTP)) throw new Error('The URI is not a TOTP one.');
  return otp;
}

/**
 * The code of the current step, once that step is past `usedStep`: the admin server takes a
 * step once (support-admin.md, H11), so a second sign-in waits for the next one.
 */
export async function freshCode(totp: TOTP, usedStep = -1): Promise<TotpCode> {
  let step = totp.counter();
  while (step <= usedStep) {
    await sleep(totp.remaining() + 50);
    step = totp.counter();
  }
  return { code: totp.generate({ timestamp: step * totp.period * 1000 }), step };
}
