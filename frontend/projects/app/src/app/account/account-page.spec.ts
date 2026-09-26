import { HttpTestingController } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';
import { Router } from '@angular/router';
import axe from 'axe-core';
import { ACCOUNT, openAccountPage, waitForEffects } from './testing/account-test-support';
import { AccountPage } from './account-page';

const SERIOUS_IMPACTS = ['serious', 'critical'];

describe('AccountPage', () => {
  afterEach(() => TestBed.inject(HttpTestingController).verify());

  it('shows the address, unverified, with a way to resend the verification email', async () => {
    const fixture = await openAccountPage(AccountPage, ACCOUNT);

    expect(fixture.nativeElement.textContent).toContain(ACCOUNT.email);
    const resend: HTMLButtonElement = fixture.nativeElement.querySelector(
      'section[aria-labelledby="account-address-heading"] button',
    );
    expect(resend).toBeTruthy();

    resend.dispatchEvent(new Event('click'));
    TestBed.inject(HttpTestingController)
      .expectOne('/api/v1/auth/verify-email/resend')
      .flush(null, { status: 204, statusText: 'No Content' });

    await waitForEffects(() => {
      expect(fixture.nativeElement.querySelector('[role="status"]')?.textContent).toBeTruthy();
    });
  });

  it('formats the storage used against the quota with Intl', async () => {
    const fixture = await openAccountPage(AccountPage, {
      ...ACCOUNT,
      storage: { usedBytes: 512_000, limitBytes: 1_000_000 },
    });

    const usage = fixture.nativeElement.querySelector(
      'section[aria-labelledby="account-usage-heading"] p',
    );
    expect(usage?.textContent?.trim()).toBe(
      `${new Intl.NumberFormat('en').format(512_000)} of ${new Intl.NumberFormat('en').format(1_000_000)} bytes used.`,
    );
  });

  it('changes the language with a CSRF header, and applies it through the preference', async () => {
    const fixture = await openAccountPage(AccountPage, ACCOUNT);

    fixture.nativeElement
      .querySelector('ion-select')
      ?.dispatchEvent(new CustomEvent('ionChange', { detail: { value: 'fr' } }));
    await fixture.whenStable();

    const http = TestBed.inject(HttpTestingController);
    const request = http.expectOne('/api/v1/account');
    expect(request.request.method).toBe('PATCH');
    expect(request.request.headers.get('X-CSRF-Token')).toBe('t0k');
    request.flush({ ...ACCOUNT, language: 'fr' });
    (await vi.waitFor(() => http.expectOne('/i18n/fr.json'))).flush({});
    await fixture.whenStable();
  });

  it('deletes the account only once the address is typed to confirm, with the password', async () => {
    const fixture = await openAccountPage(AccountPage, ACCOUNT);
    const navigation = vi.spyOn(TestBed.inject(Router), 'navigateByUrl').mockResolvedValue(true);

    fixture.nativeElement
      .querySelector('section[aria-labelledby="account-delete-heading"] button')
      ?.dispatchEvent(new Event('click'));
    await fixture.whenStable();

    const confirmButton: HTMLButtonElement = fixture.nativeElement.querySelector(
      'button[type="submit"].danger',
    );
    expect(confirmButton.disabled).toBe(true);

    fixture.nativeElement
      .querySelector('lp-password-field ion-input')
      ?.dispatchEvent(new CustomEvent('ionInput', { detail: { value: 'a very long password' } }));
    fixture.nativeElement
      .querySelector('ion-input[type="email"]')
      ?.dispatchEvent(new CustomEvent('ionInput', { detail: { value: ACCOUNT.email } }));
    await fixture.whenStable();

    expect(confirmButton.disabled).toBe(false);
    fixture.nativeElement.querySelector('form')?.dispatchEvent(new Event('submit'));
    await fixture.whenStable();

    const request = TestBed.inject(HttpTestingController).expectOne('/api/v1/account');
    expect(request.request.method).toBe('DELETE');
    expect(request.request.headers.get('X-CSRF-Token')).toBe('t0k');
    request.flush(null, { status: 204, statusText: 'No Content' });

    await waitForEffects(() => expect(navigation).toHaveBeenCalledWith('/editor'));
  });

  it('shows a wrong-password deletion error next to the password field', async () => {
    const fixture = await openAccountPage(AccountPage, ACCOUNT);

    fixture.nativeElement
      .querySelector('section[aria-labelledby="account-delete-heading"] button')
      ?.dispatchEvent(new Event('click'));
    await fixture.whenStable();
    fixture.nativeElement
      .querySelector('lp-password-field ion-input')
      ?.dispatchEvent(new CustomEvent('ionInput', { detail: { value: 'wrong password' } }));
    fixture.nativeElement
      .querySelector('ion-input[type="email"]')
      ?.dispatchEvent(new CustomEvent('ionInput', { detail: { value: ACCOUNT.email } }));
    await fixture.whenStable();
    fixture.nativeElement.querySelector('form')?.dispatchEvent(new Event('submit'));
    await fixture.whenStable();

    TestBed.inject(HttpTestingController)
      .expectOne('/api/v1/account')
      .flush(
        { code: 'auth.current_password', params: {} },
        { status: 403, statusText: 'Forbidden' },
      );

    await waitForEffects(() => {
      const passwordField = fixture.nativeElement.querySelector('lp-password-field ion-input');
      expect(passwordField?.errorText).toBeTruthy();
    });
  });

  it('signs out and returns to the editor', async () => {
    const fixture = await openAccountPage(AccountPage, ACCOUNT);
    const navigation = vi.spyOn(TestBed.inject(Router), 'navigateByUrl').mockResolvedValue(true);

    fixture.nativeElement
      .querySelector('section[aria-labelledby="account-sign-out-heading"] button')
      ?.dispatchEvent(new Event('click'));
    await fixture.whenStable();

    TestBed.inject(HttpTestingController)
      .expectOne('/api/v1/auth/sign-out')
      .flush(null, { status: 204, statusText: 'No Content' });

    await waitForEffects(() => expect(navigation).toHaveBeenCalledWith('/editor'));
  });

  it('has no serious accessibility violation', async () => {
    const fixture = await openAccountPage(AccountPage, ACCOUNT);

    const { violations } = await axe.run(fixture.nativeElement);

    expect(
      violations.filter((violation) => SERIOUS_IMPACTS.includes(violation.impact ?? '')),
    ).toEqual([]);
  });
});
