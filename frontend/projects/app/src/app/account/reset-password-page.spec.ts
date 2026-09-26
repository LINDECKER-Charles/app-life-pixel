import { HttpTestingController } from '@angular/common/http/testing';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import axe from 'axe-core';
import {
  openAccountPage,
  submitForm,
  typeInto,
  waitForEffects,
} from './testing/account-test-support';
import { ResetPasswordPage } from './reset-password-page';

const SERIOUS_IMPACTS = ['serious', 'critical'];

async function submit(fixture: ComponentFixture<ResetPasswordPage>, email: string): Promise<void> {
  typeInto(fixture.nativeElement, '#reset-password-email', email);
  await submitForm(fixture);
}

describe('ResetPasswordPage', () => {
  afterEach(() => TestBed.inject(HttpTestingController).verify());

  it('says the link is on its way, whether an account uses the address or not', async () => {
    const fixture = await openAccountPage(ResetPasswordPage);

    await submit(fixture, 'lee@example.com');
    TestBed.inject(HttpTestingController)
      .expectOne('/api/v1/auth/password-reset')
      .flush(null, { status: 204, statusText: 'No Content' });

    await waitForEffects(() => {
      expect(fixture.nativeElement.querySelector('[role="status"]')?.textContent).toContain(
        'If an account uses this address',
      );
    });
    expect(fixture.nativeElement.querySelector('form')).toBeNull();
  });

  it('asks for the address before sending, and focuses its field', async () => {
    const fixture = await openAccountPage(ResetPasswordPage);

    await submit(fixture, '   ');

    TestBed.inject(HttpTestingController).expectNone('/api/v1/auth/password-reset');
    const email = fixture.nativeElement.querySelector('#reset-password-email');
    await waitForEffects(() => expect(document.activeElement).toBe(email));
    expect(email.getAttribute('aria-invalid')).toBe('true');
  });

  it('shows a rate-limit error in the summary, keeping the address', async () => {
    const fixture = await openAccountPage(ResetPasswordPage);

    await submit(fixture, 'lee@example.com');
    TestBed.inject(HttpTestingController)
      .expectOne('/api/v1/auth/password-reset')
      .flush(
        { code: 'rate_limit.exceeded', params: { retryAfterSeconds: 30 } },
        { status: 429, statusText: 'Too Many Requests' },
      );

    await waitForEffects(() => {
      expect(fixture.nativeElement.querySelector('[role="alert"] li')?.textContent).toBeTruthy();
    });
    expect(fixture.nativeElement.querySelector('#reset-password-email').value).toBe(
      'lee@example.com',
    );
  });

  it('has no serious accessibility violation', async () => {
    const fixture = await openAccountPage(ResetPasswordPage);

    const { violations } = await axe.run(fixture.nativeElement);

    expect(
      violations.filter((violation) => SERIOUS_IMPACTS.includes(violation.impact ?? '')),
    ).toEqual([]);
  });
});
