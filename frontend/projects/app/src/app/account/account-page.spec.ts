import { HttpTestingController } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';
import { Router } from '@angular/router';
import axe from 'axe-core';
import { ACCOUNT, openAccountPage, waitForEffects } from './testing/account-test-support';
import { AccountPage } from './account-page';

const SERIOUS_IMPACTS = ['serious', 'critical'];

function section(host: HTMLElement, heading: string): HTMLElement {
  return host.querySelector(`section[aria-labelledby="${heading}"]`) as HTMLElement;
}

describe('AccountPage', () => {
  afterEach(() => TestBed.inject(HttpTestingController).verify());

  it('shows the address, unverified, with a way to resend the verification email', async () => {
    const fixture = await openAccountPage(AccountPage, ACCOUNT);
    const address = section(fixture.nativeElement, 'account-address-heading');

    expect(address.textContent).toContain(ACCOUNT.email);
    expect(address.textContent).toContain('This address is not verified yet.');
    address.querySelector('button')?.click();
    TestBed.inject(HttpTestingController)
      .expectOne('/api/v1/auth/verify-email/resend')
      .flush(null, { status: 204, statusText: 'No Content' });

    await waitForEffects(() => {
      expect(address.querySelector('[role="status"]')?.textContent).toContain(
        'A new verification email is on its way.',
      );
    });
  });

  it('says a verified address is verified, without a resend button', async () => {
    const fixture = await openAccountPage(AccountPage, { ...ACCOUNT, emailVerified: true });
    const address = section(fixture.nativeElement, 'account-address-heading');

    expect(address.textContent).toContain('This address is verified.');
    expect(address.querySelector('button')).toBeNull();
  });

  it('formats the storage used against the quota with Intl', async () => {
    const fixture = await openAccountPage(AccountPage, {
      ...ACCOUNT,
      storage: { usedBytes: 512_000, limitBytes: 1_000_000 },
    });

    const usage = section(fixture.nativeElement, 'account-usage-heading').querySelector('p');
    expect(usage?.textContent?.trim()).toBe(
      `${new Intl.NumberFormat('en').format(512_000)} of ${new Intl.NumberFormat('en').format(1_000_000)} bytes used.`,
    );
  });

  it('changes the language with a CSRF header, and applies it through the preference', async () => {
    const fixture = await openAccountPage(AccountPage, ACCOUNT);
    const select = fixture.nativeElement.querySelector('#account-language') as HTMLSelectElement;
    expect(select.labels?.[0]?.textContent?.trim()).toBe('Language');

    select.value = 'fr';
    select.dispatchEvent(new Event('change'));
    await fixture.whenStable();

    const http = TestBed.inject(HttpTestingController);
    const request = http.expectOne('/api/v1/account');
    expect(request.request.method).toBe('PATCH');
    expect(request.request.headers.get('X-CSRF-Token')).toBe('t0k');
    request.flush({ ...ACCOUNT, language: 'fr' });
    (await vi.waitFor(() => http.expectOne('/i18n/fr.json'))).flush({});
    await fixture.whenStable();
  });

  it('keeps sign-out apart from deletion, and deletion last', async () => {
    const fixture = await openAccountPage(AccountPage, ACCOUNT);

    const headings = [...fixture.nativeElement.querySelectorAll('h2')].map(
      (heading) => (heading as HTMLElement).id,
    );
    expect(headings.slice(-2)).toEqual(['account-sign-out-heading', 'account-delete-heading']);
  });

  it('signs out and returns to the editor', async () => {
    const fixture = await openAccountPage(AccountPage, ACCOUNT);
    const navigation = vi.spyOn(TestBed.inject(Router), 'navigateByUrl').mockResolvedValue(true);

    section(fixture.nativeElement, 'account-sign-out-heading').querySelector('button')?.click();
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
