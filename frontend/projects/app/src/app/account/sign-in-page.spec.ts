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
import { SignInPage } from './sign-in-page';

const SERIOUS_IMPACTS = ['serious', 'critical'];
const EMAIL = 'input[type="email"]';
const PASSWORD = 'lp-password-field input';

function fillForm(fixture: ComponentFixture<SignInPage>, email: string, password: string): void {
  typeInto(fixture.nativeElement, EMAIL, email);
  typeInto(fixture.nativeElement, PASSWORD, password);
}

function summaryItems(fixture: ComponentFixture<SignInPage>): (string | undefined)[] {
  const items = fixture.nativeElement.querySelectorAll('[role="alert"] li');
  return [...items].map((item) => (item as HTMLElement).textContent?.trim());
}

describe('SignInPage', () => {
  afterEach(() => TestBed.inject(HttpTestingController).verify());

  it('signs in and returns to the returnUrl, keeping the editor work (D37)', async () => {
    const fixture = await openAccountPage(SignInPage);
    fixture.componentRef.setInput('returnUrl', '/editor/abc');
    fillForm(fixture, 'lee@example.com', 'a very long password');
    const navigation = vi.spyOn(TestBed.inject(Router), 'navigateByUrl').mockResolvedValue(true);

    await submitForm(fixture);
    TestBed.inject(HttpTestingController)
      .expectOne('/api/v1/auth/sign-in')
      .flush({
        account: { id: 'a1', email: 'lee@example.com', emailVerified: true, language: 'en' },
        csrfToken: 't0k',
      });

    await waitForEffects(() => expect(navigation).toHaveBeenCalledWith('/editor/abc'));
  });

  it('keeps an empty form, lists both fields and focuses the first one', async () => {
    const fixture = await openAccountPage(SignInPage);

    await submitForm(fixture);

    TestBed.inject(HttpTestingController).expectNone('/api/v1/auth/sign-in');
    expect(summaryItems(fixture)).toEqual(['Enter your email address.', 'Enter your password.']);
    const email = fixture.nativeElement.querySelector(EMAIL);
    expect(email.getAttribute('aria-invalid')).toBe('true');
    const error = email.closest('.lp-field')?.querySelector('.lp-field__error');
    expect(email.getAttribute('aria-describedby')).toBe(error?.id);
    await waitForEffects(() => expect(document.activeElement).toBe(email));
  });

  it('focuses the password when only the password is missing, and clears it once typed', async () => {
    const fixture = await openAccountPage(SignInPage);
    typeInto(fixture.nativeElement, EMAIL, 'lee@example.com');

    await submitForm(fixture);

    const password = fixture.nativeElement.querySelector(PASSWORD);
    await waitForEffects(() => expect(document.activeElement).toBe(password));
    typeInto(fixture.nativeElement, PASSWORD, 'a');
    await fixture.whenStable();
    expect(password.getAttribute('aria-invalid')).toBeNull();
    expect(fixture.nativeElement.querySelector('[role="alert"]')).toBeNull();
  });

  it('shows an invalid-credentials error next to the password field, and focuses it', async () => {
    const fixture = await openAccountPage(SignInPage);
    fillForm(fixture, 'lee@example.com', 'wrong password');

    await submitForm(fixture);
    TestBed.inject(HttpTestingController)
      .expectOne('/api/v1/auth/sign-in')
      .flush(
        { code: 'auth.invalid_credentials', params: {} },
        { status: 401, statusText: 'Unauthorized' },
      );

    const password = fixture.nativeElement.querySelector(PASSWORD);
    await waitForEffects(() => expect(document.activeElement).toBe(password));
    expect(password.getAttribute('aria-invalid')).toBe('true');
    const error = password.closest('.lp-field')?.querySelector('.lp-field__error');
    expect(error?.textContent).toContain('The email address or the password is not correct.');
    expect((password as HTMLInputElement).value).toBe('wrong password');
  });

  it('shows a suspended-account error in the summary, which takes the focus', async () => {
    const fixture = await openAccountPage(SignInPage);
    fillForm(fixture, 'lee@example.com', 'a very long password');

    await submitForm(fixture);
    TestBed.inject(HttpTestingController)
      .expectOne('/api/v1/auth/sign-in')
      .flush(
        { code: 'auth.account_suspended', params: {} },
        { status: 403, statusText: 'Forbidden' },
      );

    await waitForEffects(() =>
      expect(summaryItems(fixture)).toEqual(['This account is suspended.']),
    );
    await waitForEffects(() =>
      expect(document.activeElement).toBe(fixture.nativeElement.querySelector('[role="alert"]')),
    );
  });

  it('has no serious accessibility violation, errors shown', async () => {
    const fixture = await openAccountPage(SignInPage);
    await submitForm(fixture);

    const { violations } = await axe.run(fixture.nativeElement);

    expect(
      violations.filter((violation) => SERIOUS_IMPACTS.includes(violation.impact ?? '')),
    ).toEqual([]);
  });
});
