import { HttpTestingController } from '@angular/common/http/testing';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import { Router } from '@angular/router';
import axe from 'axe-core';
import {
  openAccountPage,
  submitForm,
  typeInto,
  waitForEffects,
} from './testing/account-test-support';
import { SignUpPage } from './sign-up-page';

const SERIOUS_IMPACTS = ['serious', 'critical'];
const EMAIL = 'input[type="email"]';
const PASSWORD = 'lp-password-field input';

function fillForm(fixture: ComponentFixture<SignUpPage>, email: string, password: string): void {
  typeInto(fixture.nativeElement, EMAIL, email);
  typeInto(fixture.nativeElement, PASSWORD, password);
}

describe('SignUpPage', () => {
  afterEach(() => TestBed.inject(HttpTestingController).verify());

  it('links to the terms and the privacy policy', async () => {
    const fixture = await openAccountPage(SignUpPage);

    const links: NodeListOf<HTMLAnchorElement> =
      fixture.nativeElement.querySelectorAll('form a[href^="/legal"]');
    expect([...links].map((link) => link.getAttribute('href'))).toEqual([
      '/legal/terms',
      '/legal/privacy',
    ]);
  });

  it('creates the account in the language chosen, and returns to the returnUrl (D37)', async () => {
    const fixture = await openAccountPage(SignUpPage);
    fixture.componentRef.setInput('returnUrl', '/editor/abc');
    fillForm(fixture, 'lee@example.com', 'a very long password');
    const language = fixture.nativeElement.querySelector('form select') as HTMLSelectElement;
    language.value = 'fr';
    language.dispatchEvent(new Event('change'));
    const navigation = vi.spyOn(TestBed.inject(Router), 'navigateByUrl').mockResolvedValue(true);

    await submitForm(fixture);
    const request = TestBed.inject(HttpTestingController).expectOne('/api/v1/auth/sign-up');
    expect(request.request.body).toEqual({
      email: 'lee@example.com',
      password: 'a very long password',
      language: 'fr',
    });
    request.flush({
      account: { id: 'a1', email: 'lee@example.com', emailVerified: false, language: 'fr' },
      csrfToken: 't0k',
    });

    await waitForEffects(() => expect(navigation).toHaveBeenCalledWith('/editor/abc'));
    const http = TestBed.inject(HttpTestingController);
    (await vi.waitFor(() => http.expectOne('/i18n/fr.json'))).flush({});
  });

  it('refuses a short password before sending, and focuses it', async () => {
    const fixture = await openAccountPage(SignUpPage);
    fillForm(fixture, 'lee@example.com', 'short');

    await submitForm(fixture);

    TestBed.inject(HttpTestingController).expectNone('/api/v1/auth/sign-up');
    const password = fixture.nativeElement.querySelector(PASSWORD);
    await waitForEffects(() => expect(document.activeElement).toBe(password));
    const field = password.closest('.lp-field');
    const hint = field?.querySelector('.lp-field__hint');
    const error = field?.querySelector('.lp-field__error');
    expect(password.getAttribute('aria-describedby')).toBe(`${hint?.id} ${error?.id}`);
    expect(error?.textContent).toContain('A password holds 12 to 128 characters.');
    expect(fixture.nativeElement.querySelector(EMAIL).value).toBe('lee@example.com');
  });

  it('shows an email-taken error next to the email field, and focuses it', async () => {
    const fixture = await openAccountPage(SignUpPage);
    fillForm(fixture, 'lee@example.com', 'a very long password');

    await submitForm(fixture);
    TestBed.inject(HttpTestingController)
      .expectOne('/api/v1/auth/sign-up')
      .flush({ code: 'auth.email_taken', params: {} }, { status: 409, statusText: 'Conflict' });

    const email = fixture.nativeElement.querySelector(EMAIL);
    await waitForEffects(() => expect(document.activeElement).toBe(email));
    expect(email.getAttribute('aria-invalid')).toBe('true');
    const error = email.closest('.lp-field')?.querySelector('.lp-field__error');
    expect(error?.textContent).toContain('An account already uses this email address.');
  });

  it('has no serious accessibility violation', async () => {
    const fixture = await openAccountPage(SignUpPage);

    const { violations } = await axe.run(fixture.nativeElement);

    expect(
      violations.filter((violation) => SERIOUS_IMPACTS.includes(violation.impact ?? '')),
    ).toEqual([]);
  });
});
