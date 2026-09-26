import { HttpTestingController } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';
import axe from 'axe-core';
import { openAccountPage, waitForEffects } from './testing/account-test-support';
import { VerifyEmailPage } from './verify-email-page';

const SERIOUS_IMPACTS = ['serious', 'critical'];

describe('VerifyEmailPage', () => {
  afterEach(() => TestBed.inject(HttpTestingController).verify());

  it('asks to open the emailed link when there is no token', async () => {
    const fixture = await openAccountPage(VerifyEmailPage);

    expect(fixture.nativeElement.querySelector('p')?.textContent).toBeTruthy();
    expect(fixture.nativeElement.textContent).not.toContain('verified');
  });

  it('verifies the token and says success', async () => {
    const fixture = await openAccountPage(VerifyEmailPage);
    fixture.componentRef.setInput('token', 'a-token');
    await fixture.whenStable();

    TestBed.inject(HttpTestingController)
      .expectOne('/api/v1/auth/verify-email')
      .flush(null, { status: 204, statusText: 'No Content' });

    await waitForEffects(() => {
      expect(fixture.nativeElement.querySelector('[role="status"]')?.textContent).toBeTruthy();
    });
  });

  it('shows an invalid-token failure', async () => {
    const fixture = await openAccountPage(VerifyEmailPage);
    fixture.componentRef.setInput('token', 'a-token');
    await fixture.whenStable();

    TestBed.inject(HttpTestingController)
      .expectOne('/api/v1/auth/verify-email')
      .flush(
        { code: 'auth.token_invalid', params: {} },
        { status: 400, statusText: 'Bad Request' },
      );

    await waitForEffects(() => {
      expect(fixture.nativeElement.querySelector('[role="alert"]')?.textContent).toBeTruthy();
    });
  });

  it('has no serious accessibility violation', async () => {
    const fixture = await openAccountPage(VerifyEmailPage);

    const { violations } = await axe.run(fixture.nativeElement);

    expect(
      violations.filter((violation) => SERIOUS_IMPACTS.includes(violation.impact ?? '')),
    ).toEqual([]);
  });
});
