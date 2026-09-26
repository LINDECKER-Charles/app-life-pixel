import { HttpTestingController } from '@angular/common/http/testing';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import { Router } from '@angular/router';
import axe from 'axe-core';
import {
  ACCOUNT,
  openAccountPage,
  submitForm,
  typeInto,
  waitForEffects,
} from '../testing/account-test-support';
import { AccountDeletion } from './account-deletion';

const SERIOUS_IMPACTS = ['serious', 'critical'];
const PASSWORD = '#account-delete-password';
const CONFIRMATION = '#account-delete-confirmation';

async function open(): Promise<ComponentFixture<AccountDeletion>> {
  const fixture = await openAccountPage(AccountDeletion, ACCOUNT, { account: ACCOUNT });
  fixture.nativeElement.querySelector('button')?.click();
  await fixture.whenStable();
  return fixture;
}

function confirmButton(fixture: ComponentFixture<AccountDeletion>): HTMLButtonElement {
  return fixture.nativeElement.querySelector('button[type="submit"]');
}

describe('AccountDeletion', () => {
  afterEach(() => TestBed.inject(HttpTestingController).verify());

  it('states what is lost before anything is asked', async () => {
    const fixture = await openAccountPage(AccountDeletion, ACCOUNT, { account: ACCOUNT });

    expect(fixture.nativeElement.textContent).toContain('support requests');
    expect(fixture.nativeElement.querySelector('form')).toBeNull();
  });

  it('opens with the password focused, and closes back on its button', async () => {
    const fixture = await open();

    await waitForEffects(() =>
      expect(document.activeElement).toBe(fixture.nativeElement.querySelector(PASSWORD)),
    );
    const buttons: HTMLButtonElement[] = [...fixture.nativeElement.querySelectorAll('button')];
    buttons.find((button) => button.textContent?.trim() === 'Cancel')?.click();
    await fixture.whenStable();

    expect(fixture.nativeElement.querySelector('form')).toBeNull();
    await waitForEffects(() =>
      expect(document.activeElement?.textContent?.trim()).toBe('Delete my account'),
    );
  });

  it('deletes the account only once the address is typed to confirm, with the password', async () => {
    const fixture = await open();
    const navigation = vi.spyOn(TestBed.inject(Router), 'navigateByUrl').mockResolvedValue(true);
    expect(confirmButton(fixture).disabled).toBe(true);

    typeInto(fixture.nativeElement, PASSWORD, 'a very long password');
    typeInto(fixture.nativeElement, CONFIRMATION, ACCOUNT.email);
    await fixture.whenStable();
    expect(confirmButton(fixture).disabled).toBe(false);
    await submitForm(fixture);

    const request = TestBed.inject(HttpTestingController).expectOne('/api/v1/account');
    expect(request.request.method).toBe('DELETE');
    expect(request.request.headers.get('X-CSRF-Token')).toBe('t0k');
    request.flush(null, { status: 204, statusText: 'No Content' });

    await waitForEffects(() => expect(navigation).toHaveBeenCalledWith('/editor'));
  });

  it('asks for the password before sending, and focuses it', async () => {
    const fixture = await open();
    typeInto(fixture.nativeElement, CONFIRMATION, ACCOUNT.email);
    (document.activeElement as HTMLElement | null)?.blur();

    await submitForm(fixture);

    TestBed.inject(HttpTestingController).expectNone('/api/v1/account');
    const password = fixture.nativeElement.querySelector(PASSWORD);
    await waitForEffects(() => expect(document.activeElement).toBe(password));
    expect(password.getAttribute('aria-invalid')).toBe('true');
  });

  it('shows a wrong-password error next to the password field, and focuses it', async () => {
    const fixture = await open();
    typeInto(fixture.nativeElement, PASSWORD, 'wrong password');
    typeInto(fixture.nativeElement, CONFIRMATION, ACCOUNT.email);
    await submitForm(fixture);

    TestBed.inject(HttpTestingController)
      .expectOne('/api/v1/account')
      .flush(
        { code: 'auth.current_password', params: {} },
        { status: 403, statusText: 'Forbidden' },
      );

    await waitForEffects(() => {
      expect(fixture.nativeElement.querySelector(`${PASSWORD}-error`)?.textContent).toContain(
        'The current password is not correct.',
      );
      expect(document.activeElement).toBe(fixture.nativeElement.querySelector(PASSWORD));
    });
  });

  it('has no serious accessibility violation, the form open', async () => {
    const fixture = await open();

    const { violations } = await axe.run(fixture.nativeElement);

    expect(
      violations.filter((violation) => SERIOUS_IMPACTS.includes(violation.impact ?? '')),
    ).toEqual([]);
  });
});
