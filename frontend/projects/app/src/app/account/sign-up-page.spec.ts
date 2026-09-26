import { HttpTestingController } from '@angular/common/http/testing';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import { Router } from '@angular/router';
import axe from 'axe-core';
import { openAccountPage, waitForEffects } from './testing/account-test-support';
import { SignUpPage } from './sign-up-page';

const SERIOUS_IMPACTS = ['serious', 'critical'];

function fillIonInput(host: Element, selector: string, value: string): void {
  host.querySelector(selector)?.dispatchEvent(new CustomEvent('ionInput', { detail: { value } }));
}

function fillForm(fixture: ComponentFixture<SignUpPage>, email: string, password: string): void {
  fillIonInput(fixture.nativeElement, 'ion-input', email);
  fillIonInput(fixture.nativeElement, 'lp-password-field ion-input', password);
}

async function submit(fixture: ComponentFixture<SignUpPage>): Promise<void> {
  fixture.nativeElement.querySelector('form')?.dispatchEvent(new Event('submit'));
  await fixture.whenStable();
}

describe('SignUpPage', () => {
  afterEach(() => TestBed.inject(HttpTestingController).verify());

  it('links to the terms and the privacy policy', async () => {
    const fixture = await openAccountPage(SignUpPage);

    const links: NodeListOf<HTMLAnchorElement> = fixture.nativeElement.querySelectorAll('.legal a');
    expect([...links].map((link) => link.getAttribute('href'))).toEqual([
      '/legal/terms',
      '/legal/privacy',
    ]);
  });

  it('creates the account and returns to the returnUrl, keeping the editor work (D37)', async () => {
    const fixture = await openAccountPage(SignUpPage);
    fixture.componentRef.setInput('returnUrl', '/editor/abc');
    fillForm(fixture, 'lee@example.com', 'a very long password');
    const navigation = vi.spyOn(TestBed.inject(Router), 'navigateByUrl').mockResolvedValue(true);

    await submit(fixture);
    TestBed.inject(HttpTestingController)
      .expectOne('/api/v1/auth/sign-up')
      .flush({
        account: { id: 'a1', email: 'lee@example.com', emailVerified: false, language: 'en' },
        csrfToken: 't0k',
      });

    await waitForEffects(() => expect(navigation).toHaveBeenCalledWith('/editor/abc'));
  });

  it('shows an email-taken error next to the email field', async () => {
    const fixture = await openAccountPage(SignUpPage);
    fillForm(fixture, 'lee@example.com', 'a very long password');

    await submit(fixture);
    TestBed.inject(HttpTestingController)
      .expectOne('/api/v1/auth/sign-up')
      .flush({ code: 'auth.email_taken', params: {} }, { status: 409, statusText: 'Conflict' });

    await waitForEffects(() => {
      const emailField = fixture.nativeElement.querySelector('ion-input');
      expect(emailField?.errorText).toBeTruthy();
    });
  });

  it('shows a password-length error next to the password field', async () => {
    const fixture = await openAccountPage(SignUpPage);
    fillForm(fixture, 'lee@example.com', 'short');

    await submit(fixture);
    TestBed.inject(HttpTestingController)
      .expectOne('/api/v1/auth/sign-up')
      .flush(
        { code: 'auth.password_length', params: { min: 12, max: 128 } },
        { status: 422, statusText: 'Unprocessable Entity' },
      );

    await waitForEffects(() => {
      const passwordField = fixture.nativeElement.querySelector('lp-password-field ion-input');
      expect(passwordField?.errorText).toBeTruthy();
    });
  });

  it('has no serious accessibility violation', async () => {
    const fixture = await openAccountPage(SignUpPage);
    // `ion-select` labels its internal button asynchronously, once Stencil re-renders it.
    await waitForEffects(() => {
      const button = fixture.nativeElement
        .querySelector('ion-select')
        ?.shadowRoot?.querySelector('button');
      expect(button?.getAttribute('aria-label')).toBeTruthy();
    });

    const { violations } = await axe.run(fixture.nativeElement);

    expect(
      violations.filter((violation) => SERIOUS_IMPACTS.includes(violation.impact ?? '')),
    ).toEqual([]);
  });
});
