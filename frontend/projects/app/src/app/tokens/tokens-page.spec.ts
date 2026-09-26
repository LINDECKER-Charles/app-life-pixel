import axe from 'axe-core';
import { ApiProblem } from 'shared';
import { vi } from 'vitest';
import { ACCOUNT, openAccountPage, waitForEffects } from '../account/testing/account-test-support';
import { CREATED, mockTokens, TOKEN } from './testing/tokens-test-support';
import { TokensPage } from './tokens-page';

const SERIOUS_IMPACTS = ['serious', 'critical'];

function buttonNamed(root: ParentNode, text: string): HTMLButtonElement {
  const button = [...root.querySelectorAll<HTMLButtonElement>('button')].find(
    (candidate) => candidate.textContent?.trim() === text,
  );
  if (!button) throw new Error(`no button "${text}"`);
  return button;
}

/** Waits for the open dialog to show `selector`, and returns it: ion-modal moves it to body. */
async function shown<T extends Element>(selector: string): Promise<T> {
  return vi.waitFor(() => {
    const found = document.body.querySelector<T>(`ion-modal ${selector}`);
    if (!found) throw new Error(`${selector} not shown yet`);
    return found;
  });
}

/** Opens the page with its list shown, and the creation dialog's form. */
async function openCreationForm(): Promise<HTMLFormElement> {
  const fixture = await openAccountPage(TokensPage, ACCOUNT);
  const root: HTMLElement = fixture.nativeElement;
  await waitForEffects(() => expect(root.querySelectorAll('.tokens li')).toHaveLength(1));
  buttonNamed(root, 'Create a token').click();
  return shown<HTMLFormElement>('form');
}

function typeName(form: HTMLFormElement, name: string): void {
  const field = form.querySelector<HTMLInputElement>('#token-name');
  if (!field) throw new Error('no name field');
  field.value = name;
  field.dispatchEvent(new Event('input'));
}

/**
 * The serious violations under `root`. A closed `ion-modal` is hidden by Ionic's stylesheet, which
 * jsdom does not apply: axe would see its unnamed, empty dialog.
 */
async function seriousViolations(root: HTMLElement): Promise<axe.Result[]> {
  const { violations } = await axe.run({ include: [root], exclude: ['ion-modal.overlay-hidden'] });
  return violations.filter((violation) => SERIOUS_IMPACTS.includes(violation.impact ?? ''));
}

describe('TokensPage', () => {
  afterEach(() => document.body.replaceChildren());

  it('lists the active tokens with their prefix, scopes, dates and last use', async () => {
    const { api } = mockTokens();
    api.list.mockResolvedValueOnce([TOKEN, { ...TOKEN, id: 't3', name: 'CI', lastUsedAt: null }]);
    const fixture = await openAccountPage(TokensPage, ACCOUNT);
    const root: HTMLElement = fixture.nativeElement;

    await waitForEffects(() => expect(root.querySelectorAll('.tokens li')).toHaveLength(2));
    const [laptop, ci] = [...root.querySelectorAll('.tokens li')].map((li) => li.textContent);
    expect(laptop).toContain('Laptop');
    expect(laptop).toContain('lp_pat_AbCd…');
    expect(laptop).toContain('Read');
    expect(laptop).toContain('Export');
    expect(laptop).not.toContain('Write');
    expect(laptop).toContain('Created Sep 1, 2026');
    expect(laptop).toContain('expires Nov 30, 2026');
    expect(laptop).toContain('last used Sep 20, 2026');
    expect(ci).toContain('never used');
  });

  it('says when there is no token, and retries a failed list', async () => {
    const { api } = mockTokens();
    api.list.mockRejectedValueOnce(new ApiProblem(503, 'service.unavailable'));
    api.list.mockResolvedValueOnce([]);
    const fixture = await openAccountPage(TokensPage, ACCOUNT);
    const root: HTMLElement = fixture.nativeElement;

    await waitForEffects(() => expect(root.querySelector('[role="alert"]')).toBeTruthy());
    buttonNamed(root, 'Retry').click();

    await waitForEffects(() => expect(root.textContent).toContain('You have no active token.'));
  });

  it('creates a token with read and write for 90 days unless changed', async () => {
    const { api } = mockTokens();
    const form = await openCreationForm();
    const checked = [...form.querySelectorAll<HTMLInputElement>('input[type="checkbox"]')]
      .filter((box) => box.checked)
      .map((box) => box.value);
    expect(checked).toEqual(['read', 'write']);
    expect(form.querySelector<HTMLSelectElement>('#token-expiry')?.value).toBe('90');

    typeName(form, '  Claude Code  ');
    await vi.waitFor(() => expect(buttonNamed(form, 'Create').disabled).toBe(false));
    buttonNamed(form, 'Create').click();

    await vi.waitFor(() => expect(api.create).toHaveBeenCalledTimes(1));
    expect(api.create).toHaveBeenCalledWith({
      name: 'Claude Code',
      scopes: ['read', 'write'],
      expiresInDays: 90,
    });
  });

  it('sends the scopes and lifetime chosen', async () => {
    const { api } = mockTokens();
    const form = await openCreationForm();
    const scope = (value: string) =>
      form.querySelector<HTMLInputElement>(`input[type="checkbox"][value="${value}"]`);
    scope('write')?.click();
    scope('export')?.click();
    const expiry = form.querySelector<HTMLSelectElement>('#token-expiry');
    if (!expiry) throw new Error('no expiry field');
    expiry.value = '365';
    expiry.dispatchEvent(new Event('change'));
    typeName(form, 'CI');
    await vi.waitFor(() => expect(buttonNamed(form, 'Create').disabled).toBe(false));
    buttonNamed(form, 'Create').click();

    await vi.waitFor(() =>
      expect(api.create).toHaveBeenCalledWith({
        name: 'CI',
        scopes: ['read', 'export'],
        expiresInDays: 365,
      }),
    );
  });

  it('refuses a token without a scope before sending it', async () => {
    const { api } = mockTokens();
    const form = await openCreationForm();
    typeName(form, 'CI');
    for (const value of ['read', 'write']) {
      form.querySelector<HTMLInputElement>(`input[type="checkbox"][value="${value}"]`)?.click();
    }

    await vi.waitFor(() => expect(form.textContent).toContain('Choose at least one permission.'));
    expect(buttonNamed(form, 'Create').disabled).toBe(true);
    expect(api.create).not.toHaveBeenCalled();
  });

  it('shows the secret once, with the command to copy, and lists the token first', async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    Object.defineProperty(navigator, 'clipboard', { value: { writeText }, configurable: true });
    mockTokens();
    const form = await openCreationForm();
    typeName(form, 'Claude Code');
    await vi.waitFor(() => expect(buttonNamed(form, 'Create').disabled).toBe(false));
    buttonNamed(form, 'Create').click();

    const code = await shown<HTMLElement>('lp-created-token');
    expect(code.textContent).toContain(CREATED.token);
    const command =
      'claude mcp add --transport http life-pixel https://life-pixel.app/mcp ' +
      `--header "Authorization: Bearer ${CREATED.token}"`;
    expect(code.textContent).toContain(command);
    buttonNamed(code, 'Copy the command').click();
    await vi.waitFor(() => expect(writeText).toHaveBeenCalledWith(command));
    buttonNamed(code, 'Copy the token').click();
    await vi.waitFor(() => expect(writeText).toHaveBeenLastCalledWith(CREATED.token));
    const names = [...document.querySelectorAll('.tokens .name')].map((name) => name.textContent);
    expect(names).toEqual(['Claude Code', 'Laptop']);

    buttonNamed(code, 'Done').click();
    await vi.waitFor(() => expect(document.body.textContent).not.toContain(CREATED.token));
  });

  it('shows the server refusing the name next to it', async () => {
    const { api } = mockTokens();
    api.create.mockRejectedValueOnce(new ApiProblem(422, 'token.name', { max: 60 }));
    const form = await openCreationForm();
    typeName(form, 'CI');
    await vi.waitFor(() => expect(buttonNamed(form, 'Create').disabled).toBe(false));
    buttonNamed(form, 'Create').click();

    await vi.waitFor(() =>
      expect(form.querySelector('.field [role="alert"]')?.textContent).toContain('60'),
    );
  });

  it('shows the limit of active tokens above the form', async () => {
    const { api } = mockTokens();
    api.create.mockRejectedValueOnce(new ApiProblem(409, 'token.limit', { max: 20 }));
    const form = await openCreationForm();
    typeName(form, 'CI');
    await vi.waitFor(() => expect(buttonNamed(form, 'Create').disabled).toBe(false));
    buttonNamed(form, 'Create').click();

    await vi.waitFor(() =>
      expect(form.querySelector('.banner')?.textContent).toContain('20 active tokens'),
    );
  });

  it('revokes a token once confirmed, and keeps it otherwise', async () => {
    const { api, revocation } = mockTokens();
    revocation.confirm.mockResolvedValueOnce(false);
    const fixture = await openAccountPage(TokensPage, ACCOUNT);
    const root: HTMLElement = fixture.nativeElement;
    await waitForEffects(() => expect(root.querySelectorAll('.tokens li')).toHaveLength(1));

    buttonNamed(root, 'Revoke').click();
    await vi.waitFor(() => expect(revocation.confirm).toHaveBeenCalledWith('Laptop'));
    expect(api.revoke).not.toHaveBeenCalled();
    buttonNamed(root, 'Revoke').click();

    await waitForEffects(() => expect(root.querySelectorAll('.tokens li')).toHaveLength(0));
    expect(api.revoke).toHaveBeenCalledWith('t1');
  });

  it('shows a refused revocation, keeping the token', async () => {
    const { api } = mockTokens();
    api.revoke.mockRejectedValueOnce(new ApiProblem(404, 'token.not_found'));
    const fixture = await openAccountPage(TokensPage, ACCOUNT);
    const root: HTMLElement = fixture.nativeElement;
    await waitForEffects(() => expect(root.querySelectorAll('.tokens li')).toHaveLength(1));

    buttonNamed(root, 'Revoke').click();

    await waitForEffects(() =>
      expect(root.querySelector('[role="alert"]')?.textContent).toContain('already revoked'),
    );
    expect(root.querySelectorAll('.tokens li')).toHaveLength(1);
  });

  it('has no serious accessibility violation, listing tokens', async () => {
    mockTokens();
    const fixture = await openAccountPage(TokensPage, ACCOUNT);
    const root: HTMLElement = fixture.nativeElement;
    await waitForEffects(() => expect(root.querySelector('.tokens li')).toBeTruthy());

    expect(await seriousViolations(root)).toEqual([]);
  });

  it('has no serious accessibility violation, creating a token and showing its secret', async () => {
    mockTokens();
    const form = await openCreationForm();
    expect(await seriousViolations(document.body)).toEqual([]);
    typeName(form, 'Claude Code');
    await vi.waitFor(() => expect(buttonNamed(form, 'Create').disabled).toBe(false));
    buttonNamed(form, 'Create').click();
    await shown('lp-created-token');

    expect(await seriousViolations(document.body)).toEqual([]);
  });
});
