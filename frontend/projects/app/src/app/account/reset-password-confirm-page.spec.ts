import { HttpTestingController } from '@angular/common/http/testing';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import axe from 'axe-core';
import {
  ACCOUNT,
  openAccountPage,
  submitForm,
  typeInto,
  waitForEffects,
} from './testing/account-test-support';
import { ResetPasswordConfirmPage } from './reset-password-confirm-page';

const SERIOUS_IMPACTS = ['serious', 'critical'];
const FIELD = 'lp-password-field input';

async function submit(
  fixture: ComponentFixture<ResetPasswordConfirmPage>,
  password: string,
): Promise<void> {
  typeInto(fixture.nativeElement, FIELD, password);
  await submitForm(fixture);
}

async function openWithToken(
  account: typeof ACCOUNT | null = null,
): Promise<ComponentFixture<ResetPasswordConfirmPage>> {
  return openAccountPage(ResetPasswordConfirmPage, account, { token: 'a-token' });
}

describe('ResetPasswordConfirmPage', () => {
  afterEach(() => TestBed.inject(HttpTestingController).verify());

  it('sets the new password and ends every session, this browser included (H7)', async () => {
    const fixture = await openWithToken({ ...ACCOUNT, emailVerified: true });

    await submit(fixture, 'a new long password');
    TestBed.inject(HttpTestingController)
      .expectOne('/api/v1/auth/password-reset/confirm')
      .flush(null, { status: 204, statusText: 'No Content' });

    await waitForEffects(() => {
      expect(fixture.nativeElement.querySelector('[role="status"]')?.textContent).toContain(
        'Your password is set.',
      );
    });
  });

  it('refuses a short password before sending, next to the field, and focuses it', async () => {
    const fixture = await openWithToken();

    await submit(fixture, 'short');

    TestBed.inject(HttpTestingController).expectNone('/api/v1/auth/password-reset/confirm');
    const password = fixture.nativeElement.querySelector(FIELD);
    await waitForEffects(() => expect(document.activeElement).toBe(password));
    const error = password.closest('.lp-field')?.querySelector('.lp-field__error');
    expect(error?.textContent).toContain('A password holds 12 to 128 characters.');
  });

  it('shows an invalid-token error in the summary, which takes the focus', async () => {
    const fixture = await openWithToken();

    await submit(fixture, 'a new long password');
    TestBed.inject(HttpTestingController)
      .expectOne('/api/v1/auth/password-reset/confirm')
      .flush(
        { code: 'auth.token_invalid', params: {} },
        { status: 400, statusText: 'Bad Request' },
      );

    await waitForEffects(() => {
      const summary = fixture.nativeElement.querySelector('[role="alert"]');
      expect(summary?.textContent).toContain('This link is invalid, already used or expired.');
      expect(document.activeElement).toBe(summary);
    });
  });

  it('has no serious accessibility violation', async () => {
    const fixture = await openWithToken();

    const { violations } = await axe.run(fixture.nativeElement);

    expect(
      violations.filter((violation) => SERIOUS_IMPACTS.includes(violation.impact ?? '')),
    ).toEqual([]);
  });
});
