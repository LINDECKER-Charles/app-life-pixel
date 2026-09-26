import axe from 'axe-core';
import { ApiProblem } from 'shared';
import { ACCOUNT, openAccountPage, waitForEffects } from '../account/testing/account-test-support';
import { SupportPage } from './support-page';
import { SUPPORT_LIMITS } from './support-limits';
import { mockSupportApi, THREAD } from './testing/support-test-support';

const SERIOUS_IMPACTS = ['serious', 'critical'];
const SUMMARY = {
  id: 'r0',
  category: 'account',
  status: 'waiting_for_user',
  hasScreenshot: false,
  createdAt: '2026-09-20T10:00:00Z',
  updatedAt: '2026-09-21T10:00:00Z',
};

function element<T extends Element>(root: HTMLElement, selector: string): T {
  const found = root.querySelector<T>(selector);
  expect(found).toBeTruthy();
  return found as T;
}

/** Picks `category` and types `message` in the new request's form. */
async function fill(root: HTMLElement, category: string, message: string): Promise<void> {
  element(root, 'ion-select').dispatchEvent(
    new CustomEvent('ionChange', { detail: { value: category } }),
  );
  const field = element<HTMLTextAreaElement>(root, 'textarea');
  field.value = message;
  field.dispatchEvent(new Event('input'));
}

/** Picks `file` in the screenshot's file picker. */
function pick(root: HTMLElement, file: File): void {
  const picker = element<HTMLInputElement>(root, 'input[type="file"]');
  Object.defineProperty(picker, 'files', { value: [file], configurable: true });
  picker.dispatchEvent(new Event('change'));
}

describe('SupportPage', () => {
  it('explains to a signed-out visitor why support needs an account', async () => {
    const api = mockSupportApi();
    const fixture = await openAccountPage(SupportPage, null);

    const root: HTMLElement = fixture.nativeElement;
    expect(root.textContent).toContain('Support requests need an account');
    const signIn = element<HTMLAnchorElement>(root, 'a[href^="/sign-in"]');
    expect(signIn.getAttribute('href')).toBe('/sign-in?returnUrl=%2Fsupport');
    expect(api.list).not.toHaveBeenCalled();
  });

  it('lists the requests, and the next page on demand', async () => {
    const api = mockSupportApi();
    api.list.mockResolvedValueOnce({ items: [SUMMARY], nextCursor: 'c1' });
    api.list.mockResolvedValueOnce({ items: [{ ...SUMMARY, id: 'r9' }], nextCursor: null });
    const fixture = await openAccountPage(SupportPage, ACCOUNT);
    const root: HTMLElement = fixture.nativeElement;

    await waitForEffects(() => expect(root.querySelectorAll('.requests li')).toHaveLength(1));
    expect(root.textContent).toContain('Waiting for your reply');
    expect(element(root, '.requests a').getAttribute('href')).toBe('/support/r0');
    const more = [...root.querySelectorAll('button')].find((b) =>
      b.textContent?.includes('Show more'),
    );
    more?.dispatchEvent(new Event('click'));

    await waitForEffects(() => expect(root.querySelectorAll('.requests li')).toHaveLength(2));
    expect(api.list).toHaveBeenLastCalledWith('c1');
  });

  it('sends a new request with its context, and shows it first', async () => {
    const api = mockSupportApi();
    const fixture = await openAccountPage(SupportPage, ACCOUNT);
    const root: HTMLElement = fixture.nativeElement;
    const submit = element<HTMLButtonElement>(root, 'button[type="submit"]');
    expect(submit.disabled).toBe(true);

    await fill(root, 'bug', 'The canvas stays blank.');
    await fixture.whenStable();
    expect(element(root, '#support-message-counter').textContent?.trim()).toBe(
      `23 of ${new Intl.NumberFormat('en').format(SUPPORT_LIMITS.messageMaxChars)} characters`,
    );
    expect(submit.disabled).toBe(false);
    element(root, 'form').dispatchEvent(new Event('submit'));

    await waitForEffects(() => expect(root.querySelector('[role="status"]')).toBeTruthy());
    expect(api.create).toHaveBeenCalledWith({
      category: 'bug',
      message: 'The canvas stays blank.',
      context: {
        appVersion: expect.any(String),
        platform: 'web',
        language: 'en',
        screen: '/editor',
      },
      screenshot: undefined,
    });
    expect(element(root, '.requests a').getAttribute('href')).toBe(`/support/${THREAD.id}`);
  });

  it('refuses a screenshot of the wrong type or size before any upload', async () => {
    const api = mockSupportApi();
    const fixture = await openAccountPage(SupportPage, ACCOUNT);
    const root: HTMLElement = fixture.nativeElement;

    pick(root, new File(['GIF89a'], 'capture.gif', { type: 'image/gif' }));
    await waitForEffects(() =>
      expect(root.querySelector('.screenshot [role="alert"]')?.textContent).toContain(
        'PNG or JPEG',
      ),
    );
    const heavy = new File(['x'], 'capture.png', { type: 'image/png' });
    Object.defineProperty(heavy, 'size', { value: SUPPORT_LIMITS.screenshotMaxBytes + 1 });
    pick(root, heavy);
    await waitForEffects(() =>
      expect(root.querySelector('.screenshot [role="alert"]')?.textContent).toContain('too large'),
    );
    const fine = new File(['png'], 'capture.png', { type: 'image/png' });
    pick(root, fine);
    await fill(root, 'bug', 'See the capture');
    await fixture.whenStable();
    element(root, 'form').dispatchEvent(new Event('submit'));

    await waitForEffects(() => expect(api.create).toHaveBeenCalledTimes(1));
    expect(api.create.mock.calls[0][0].screenshot).toBe(fine);
  });

  it('shows the server refusing the screenshot next to it', async () => {
    const api = mockSupportApi();
    api.create.mockRejectedValueOnce(
      new ApiProblem(422, 'support.screenshot', { maxSide: 4096, maxBytes: 5_242_880 }),
    );
    const fixture = await openAccountPage(SupportPage, ACCOUNT);
    const root: HTMLElement = fixture.nativeElement;

    await fill(root, 'other', 'Hello');
    await fixture.whenStable();
    element(root, 'form').dispatchEvent(new Event('submit'));

    await waitForEffects(() =>
      expect(root.querySelector('.screenshot [role="alert"]')?.textContent).toContain('4,096'),
    );
  });

  it('has no serious accessibility violation, signed in', async () => {
    mockSupportApi().list.mockResolvedValue({ items: [SUMMARY], nextCursor: 'c1' });
    const fixture = await openAccountPage(SupportPage, ACCOUNT);
    await waitForEffects(() =>
      expect(fixture.nativeElement.querySelector('.requests li')).toBeTruthy(),
    );

    const { violations } = await axe.run(fixture.nativeElement);

    expect(
      violations.filter((violation) => SERIOUS_IMPACTS.includes(violation.impact ?? '')),
    ).toEqual([]);
  });

  it('has no serious accessibility violation, signed out', async () => {
    mockSupportApi();
    const fixture = await openAccountPage(SupportPage, null);

    const { violations } = await axe.run(fixture.nativeElement);

    expect(
      violations.filter((violation) => SERIOUS_IMPACTS.includes(violation.impact ?? '')),
    ).toEqual([]);
  });
});
