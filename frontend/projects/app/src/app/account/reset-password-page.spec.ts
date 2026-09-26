import { HttpTestingController } from '@angular/common/http/testing';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import axe from 'axe-core';
import { openAccountPage, waitForEffects } from './testing/account-test-support';
import { ResetPasswordPage } from './reset-password-page';

const SERIOUS_IMPACTS = ['serious', 'critical'];

async function submit(fixture: ComponentFixture<ResetPasswordPage>, email: string): Promise<void> {
  fixture.nativeElement
    .querySelector('ion-input')
    ?.dispatchEvent(new CustomEvent('ionInput', { detail: { value: email } }));
  fixture.nativeElement.querySelector('form')?.dispatchEvent(new Event('submit'));
  await fixture.whenStable();
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
      expect(fixture.nativeElement.querySelector('[role="status"]')?.textContent).toBeTruthy();
    });
  });

  it('shows a rate-limit error as a form banner', async () => {
    const fixture = await openAccountPage(ResetPasswordPage);

    await submit(fixture, 'lee@example.com');
    TestBed.inject(HttpTestingController)
      .expectOne('/api/v1/auth/password-reset')
      .flush(
        { code: 'rate_limit.exceeded', params: { retryAfterSeconds: 30 } },
        { status: 429, statusText: 'Too Many Requests' },
      );

    await waitForEffects(() => {
      expect(fixture.nativeElement.querySelector('.banner')?.textContent).toBeTruthy();
    });
  });

  it('has no serious accessibility violation', async () => {
    const fixture = await openAccountPage(ResetPasswordPage);

    const { violations } = await axe.run(fixture.nativeElement);

    expect(
      violations.filter((violation) => SERIOUS_IMPACTS.includes(violation.impact ?? '')),
    ).toEqual([]);
  });
});
