import { HttpTestingController } from '@angular/common/http/testing';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import { Router } from '@angular/router';
import axe from 'axe-core';
import { openAccountPage, waitForEffects } from './testing/account-test-support';
import { SignInPage } from './sign-in-page';

const SERIOUS_IMPACTS = ['serious', 'critical'];

function fillIonInput(host: Element, selector: string, value: string): void {
  host.querySelector(selector)?.dispatchEvent(new CustomEvent('ionInput', { detail: { value } }));
}

function fillForm(fixture: ComponentFixture<SignInPage>, email: string, password: string): void {
  fillIonInput(fixture.nativeElement, 'ion-input', email);
  fillIonInput(fixture.nativeElement, 'lp-password-field ion-input', password);
}

async function submit(fixture: ComponentFixture<SignInPage>): Promise<void> {
  fixture.nativeElement.querySelector('form')?.dispatchEvent(new Event('submit'));
  await fixture.whenStable();
}

describe('SignInPage', () => {
  afterEach(() => TestBed.inject(HttpTestingController).verify());

  it('signs in and returns to the returnUrl, keeping the editor work (D37)', async () => {
    const fixture = await openAccountPage(SignInPage);
    fixture.componentRef.setInput('returnUrl', '/editor/abc');
    fillForm(fixture, 'lee@example.com', 'a very long password');
    const navigation = vi.spyOn(TestBed.inject(Router), 'navigateByUrl').mockResolvedValue(true);

    await submit(fixture);
    TestBed.inject(HttpTestingController)
      .expectOne('/api/v1/auth/sign-in')
      .flush({
        account: { id: 'a1', email: 'lee@example.com', emailVerified: true, language: 'en' },
        csrfToken: 't0k',
      });

    await waitForEffects(() => expect(navigation).toHaveBeenCalledWith('/editor/abc'));
  });

  it('shows an invalid-credentials error next to the password field', async () => {
    const fixture = await openAccountPage(SignInPage);
    fillForm(fixture, 'lee@example.com', 'wrong password');

    await submit(fixture);
    TestBed.inject(HttpTestingController)
      .expectOne('/api/v1/auth/sign-in')
      .flush(
        { code: 'auth.invalid_credentials', params: {} },
        { status: 401, statusText: 'Unauthorized' },
      );

    await waitForEffects(() => {
      const passwordField = fixture.nativeElement.querySelector('lp-password-field ion-input');
      expect(passwordField?.errorText).toBeTruthy();
    });
  });

  it('shows a suspended-account error as a form banner', async () => {
    const fixture = await openAccountPage(SignInPage);
    fillForm(fixture, 'lee@example.com', 'a very long password');

    await submit(fixture);
    TestBed.inject(HttpTestingController)
      .expectOne('/api/v1/auth/sign-in')
      .flush(
        { code: 'auth.account_suspended', params: {} },
        { status: 403, statusText: 'Forbidden' },
      );

    await waitForEffects(() => {
      const banner = fixture.nativeElement.querySelector('.banner');
      expect(banner?.textContent).toContain('suspended');
    });
  });

  it('has no serious accessibility violation', async () => {
    const fixture = await openAccountPage(SignInPage);

    const { violations } = await axe.run(fixture.nativeElement);

    expect(
      violations.filter((violation) => SERIOUS_IMPACTS.includes(violation.impact ?? '')),
    ).toEqual([]);
  });
});
