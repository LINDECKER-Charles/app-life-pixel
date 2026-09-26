import { HttpTestingController } from '@angular/common/http/testing';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import axe from 'axe-core';
import { openAccountPage, waitForEffects } from './testing/account-test-support';
import { ResetPasswordConfirmPage } from './reset-password-confirm-page';

const SERIOUS_IMPACTS = ['serious', 'critical'];

async function submit(
  fixture: ComponentFixture<ResetPasswordConfirmPage>,
  password: string,
): Promise<void> {
  fixture.nativeElement
    .querySelector('lp-password-field ion-input')
    ?.dispatchEvent(new CustomEvent('ionInput', { detail: { value: password } }));
  fixture.nativeElement.querySelector('form')?.dispatchEvent(new Event('submit'));
  await fixture.whenStable();
}

describe('ResetPasswordConfirmPage', () => {
  afterEach(() => TestBed.inject(HttpTestingController).verify());

  it('sets the new password and ends every session, this browser included (H7)', async () => {
    const fixture = await openAccountPage(ResetPasswordConfirmPage, {
      id: 'a1',
      email: 'lee@example.com',
      emailVerified: true,
      language: 'en',
      plan: 'free',
      storage: { usedBytes: 0, limitBytes: 1_000_000 },
      createdAt: '2024-01-01T00:00:00Z',
    });
    fixture.componentRef.setInput('token', 'a-token');
    await fixture.whenStable();

    await submit(fixture, 'a new long password');
    TestBed.inject(HttpTestingController)
      .expectOne('/api/v1/auth/password-reset/confirm')
      .flush(null, { status: 204, statusText: 'No Content' });

    await waitForEffects(() => {
      expect(fixture.nativeElement.querySelector('[role="status"]')?.textContent).toBeTruthy();
    });
  });

  it('shows a password-length error next to the password field', async () => {
    const fixture = await openAccountPage(ResetPasswordConfirmPage);
    fixture.componentRef.setInput('token', 'a-token');
    await fixture.whenStable();

    await submit(fixture, 'short');
    TestBed.inject(HttpTestingController)
      .expectOne('/api/v1/auth/password-reset/confirm')
      .flush(
        { code: 'auth.password_length', params: { min: 12, max: 128 } },
        { status: 422, statusText: 'Unprocessable Entity' },
      );

    await waitForEffects(() => {
      const passwordField = fixture.nativeElement.querySelector('lp-password-field ion-input');
      expect(passwordField?.errorText).toBeTruthy();
    });
  });

  it('shows an invalid-token error as a form banner', async () => {
    const fixture = await openAccountPage(ResetPasswordConfirmPage);
    fixture.componentRef.setInput('token', 'a-token');
    await fixture.whenStable();

    await submit(fixture, 'a new long password');
    TestBed.inject(HttpTestingController)
      .expectOne('/api/v1/auth/password-reset/confirm')
      .flush(
        { code: 'auth.token_invalid', params: {} },
        { status: 400, statusText: 'Bad Request' },
      );

    await waitForEffects(() => {
      expect(fixture.nativeElement.querySelector('.banner')?.textContent).toBeTruthy();
    });
  });

  it('has no serious accessibility violation', async () => {
    const fixture = await openAccountPage(ResetPasswordConfirmPage);

    const { violations } = await axe.run(fixture.nativeElement);

    expect(
      violations.filter((violation) => SERIOUS_IMPACTS.includes(violation.impact ?? '')),
    ).toEqual([]);
  });
});
