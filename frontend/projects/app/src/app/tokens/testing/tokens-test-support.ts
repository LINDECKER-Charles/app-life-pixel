import { TestBed } from '@angular/core/testing';
import { type AccessTokenSummary, type CreatedAccessToken, TokensApi } from 'shared';
import { vi } from 'vitest';
import { TokenRevocation } from '../token-revocation';

/** An active token of the account, as the server lists it. */
export const TOKEN: AccessTokenSummary = {
  id: 't1',
  name: 'Laptop',
  prefix: 'lp_pat_AbCd',
  scopes: ['read', 'export'],
  createdAt: '2026-09-01T10:00:00Z',
  expiresAt: '2026-11-30T10:00:00Z',
  lastUsedAt: '2026-09-20T08:30:00Z',
};

/** A token just created, its secret in it. */
export const CREATED: CreatedAccessToken = {
  id: 't2',
  name: 'Claude Code',
  prefix: 'lp_pat_WxYz',
  scopes: ['read', 'write'],
  createdAt: '2026-09-26T10:00:00Z',
  expiresAt: '2026-12-25T10:00:00Z',
  lastUsedAt: null,
  token: `lp_pat_WxYz${'q'.repeat(39)}`,
  mcp: { serverName: 'life-pixel', url: 'https://life-pixel.app/mcp' },
};

/** A `TokensApi` whose every method is a mock, listing `TOKEN` by default. */
export type MockTokensApi = Record<'list' | 'create' | 'revoke', ReturnType<typeof vi.fn>>;

/** The revocation's confirmation, answering yes by default. */
export type MockRevocation = Record<'confirm', ReturnType<typeof vi.fn>>;

/**
 * The control that the `<label>` reading `text` names through its `for`, as assistive technology
 * finds it: throws when the label names no `type` control.
 */
export function labelledControl<T extends HTMLElement>(
  root: ParentNode,
  text: string,
  type: new () => T,
): T {
  const label = [...root.querySelectorAll('label')].find(
    (candidate) => candidate.textContent?.trim() === text,
  );
  const control = label?.control;
  if (!(control instanceof type)) throw new Error(`no ${type.name} labelled "${text}"`);
  return control;
}

/** What `control`'s `aria-describedby` reads out: the text of each element it names, in order. */
export function description(control: Element): string {
  const ids = control.getAttribute('aria-describedby')?.split(' ') ?? [];
  return ids.map((id) => control.ownerDocument.getElementById(id)?.textContent?.trim()).join(' ');
}

/**
 * Provides a mocked `TokensApi` and revocation confirmation to the next `openAccountPage`: call it
 * first, before the testing module is instantiated.
 */
export function mockTokens(): { api: MockTokensApi; revocation: MockRevocation } {
  const api: MockTokensApi = {
    list: vi.fn().mockResolvedValue([TOKEN]),
    create: vi.fn().mockResolvedValue(CREATED),
    revoke: vi.fn().mockResolvedValue(undefined),
  };
  const revocation: MockRevocation = { confirm: vi.fn().mockResolvedValue(true) };
  TestBed.configureTestingModule({
    providers: [
      { provide: TokensApi, useValue: api },
      { provide: TokenRevocation, useValue: revocation },
    ],
  });
  return { api, revocation };
}
