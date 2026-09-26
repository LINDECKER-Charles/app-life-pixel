import { randomBytes, randomInt } from 'node:crypto';

/** The domain of every recipient of the suite: reserved, it never reaches a real mailbox. */
export const TEST_MAIL_DOMAIN = 'test.life-pixel.invalid';

/** Someone the journeys sign up as: an address of their own, and where they connect from. */
export interface Person {
  readonly email: string;
  readonly password: string;
  /** The address the person's requests come from, sent in `X-Forwarded-For`. */
  readonly clientAddress: string;
}

function randomHex(bytes: number): string {
  return randomBytes(bytes).toString('hex');
}

/** A new recipient at the test domain, such as `lp-e2e-visitor-1a2b…@test.life-pixel.invalid`. */
export function randomEmail(role: string): string {
  return `lp-e2e-${role}-${randomHex(8)}@${TEST_MAIL_DOMAIN}`;
}

/** A password of 24 characters: within the 12 to 128 the accounts accept. */
export function randomPassword(): string {
  return `Pixel-${randomHex(9)}`;
}

/** An address of 198.18.0.0/15, the range set aside for tests between networks (RFC 2544). */
function randomClientAddress(): string {
  return `198.${18 + randomInt(2)}.${randomInt(256)}.${1 + randomInt(254)}`;
}

/** A new person, who has no account yet. */
export function newPerson(role: string): Person {
  return {
    email: randomEmail(role),
    password: randomPassword(),
    clientAddress: randomClientAddress(),
  };
}
