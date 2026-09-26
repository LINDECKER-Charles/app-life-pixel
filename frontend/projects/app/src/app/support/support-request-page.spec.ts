import axe from 'axe-core';
import { ApiProblem } from 'shared';
import { ACCOUNT, openAccountPage, waitForEffects } from '../account/testing/account-test-support';
import { SupportRequestPage } from './support-request-page';
import { mockSupportApi, TEAM_MESSAGE, THREAD } from './testing/support-test-support';

const SERIOUS_IMPACTS = ['serious', 'critical'];
const ANSWERED = {
  ...THREAD,
  status: 'waiting_for_user',
  messages: [...THREAD.messages, TEAM_MESSAGE],
};

/** Opens the thread of `THREAD`, signed in unless `account` is `null`. */
function open(account: typeof ACCOUNT | null = ACCOUNT) {
  return openAccountPage(SupportRequestPage, account, { requestId: THREAD.id });
}

describe('SupportRequestPage', () => {
  it('shows the thread oldest first, the team set apart', async () => {
    mockSupportApi().get.mockResolvedValue(ANSWERED);
    const fixture = await open();
    const root: HTMLElement = fixture.nativeElement;

    await waitForEffects(() => expect(root.querySelectorAll('.messages li')).toHaveLength(2));
    const messages = [...root.querySelectorAll('.messages li')];
    expect(messages[0].textContent).toContain('You');
    expect(messages[0].textContent).toContain('The canvas stays blank.');
    expect(messages[1].classList).toContain('team');
    expect(messages[1].textContent).toContain('Life Pixel support');
    expect(root.textContent).toContain('Waiting for your reply');
  });

  it('sends a reply, then shows the thread again', async () => {
    const api = mockSupportApi();
    api.get.mockResolvedValueOnce(ANSWERED);
    const fixture = await open();
    const root: HTMLElement = fixture.nativeElement;
    await waitForEffects(() => expect(root.querySelector('form')).toBeTruthy());

    const field = root.querySelector('textarea') as HTMLTextAreaElement;
    field.value = 'Still blank';
    field.dispatchEvent(new Event('input'));
    await fixture.whenStable();
    root.querySelector('form')?.dispatchEvent(new Event('submit'));

    await waitForEffects(() => expect(api.get).toHaveBeenCalledTimes(2));
    expect(api.reply).toHaveBeenCalledWith(THREAD.id, 'Still blank');
  });

  it('offers no reply on a closed request', async () => {
    mockSupportApi().get.mockResolvedValue({ ...THREAD, status: 'closed' });
    const fixture = await open();
    const root: HTMLElement = fixture.nativeElement;

    await waitForEffects(() => expect(root.textContent).toContain('This request is closed'));
    expect(root.querySelector('form')).toBeNull();
  });

  it('says so when the request is not the account’s', async () => {
    mockSupportApi().get.mockRejectedValue(new ApiProblem(404, 'support.request_not_found'));
    const fixture = await open();

    await waitForEffects(() =>
      expect(fixture.nativeElement.querySelector('[role="alert"]')?.textContent).toContain(
        'does not exist',
      ),
    );
  });

  it('explains to a signed-out visitor, bringing them back here', async () => {
    const api = mockSupportApi();
    const fixture = await open(null);

    const signIn = fixture.nativeElement.querySelector('a[href^="/sign-in"]');
    expect(signIn?.getAttribute('href')).toBe(`/sign-in?returnUrl=%2Fsupport%2F${THREAD.id}`);
    expect(api.get).not.toHaveBeenCalled();
  });

  it('has no serious accessibility violation', async () => {
    mockSupportApi().get.mockResolvedValue(ANSWERED);
    const fixture = await open();
    await waitForEffects(() => expect(fixture.nativeElement.querySelector('form')).toBeTruthy());

    const { violations } = await axe.run(fixture.nativeElement);

    expect(
      violations.filter((violation) => SERIOUS_IMPACTS.includes(violation.impact ?? '')),
    ).toEqual([]);
  });
});
