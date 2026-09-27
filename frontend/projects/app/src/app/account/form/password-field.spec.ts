import { HttpTestingController } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';
import { openAccountPage, waitForEffects } from '../testing/account-test-support';
import { PasswordField } from './password-field';

const INPUTS = { fieldId: 'test-password', label: 'Password', hint: 'At least 12 characters.' };

function nativeInput(fixture: { nativeElement: HTMLElement }): HTMLInputElement {
  return fixture.nativeElement.querySelector('input') as HTMLInputElement;
}

describe('PasswordField', () => {
  afterEach(() => TestBed.inject(HttpTestingController).verify());

  it('is named by its visible label', async () => {
    const fixture = await openAccountPage(PasswordField, null, INPUTS);

    const input = nativeInput(fixture);
    expect(input.labels?.[0]?.textContent?.trim()).toBe('Password');
    expect(input.id).toBe(INPUTS.fieldId);
  });

  it('hides the password by default, and shows it once toggled', async () => {
    const fixture = await openAccountPage(PasswordField, null, INPUTS);

    expect(nativeInput(fixture).type).toBe('password');
    const toggle = fixture.nativeElement.querySelector('button') as HTMLButtonElement;
    expect(toggle.textContent?.trim()).toBe('Show password');

    toggle.click();

    await waitForEffects(() => expect(nativeInput(fixture).type).toBe('text'));
    expect(toggle.textContent?.trim()).toBe('Hide password');
  });

  it('emits the value typed', async () => {
    const fixture = await openAccountPage(PasswordField, null, INPUTS);
    let emitted = '';
    fixture.componentInstance.valueChange.subscribe((value) => (emitted = value));

    nativeInput(fixture).value = 'a secret';
    nativeInput(fixture).dispatchEvent(new Event('input'));

    expect(emitted).toBe('a secret');
  });

  it('ties its hint and its error to the field, and marks it invalid', async () => {
    const fixture = await openAccountPage(PasswordField, null, {
      ...INPUTS,
      error: { key: 'errors.auth.invalid_credentials', params: {} },
    });

    const input = nativeInput(fixture);
    const host: HTMLElement = fixture.nativeElement;
    const descriptions = input
      .getAttribute('aria-describedby')
      ?.split(' ')
      .map((id) => host.querySelector(`[id="${id}"]`)?.textContent?.trim());
    expect(input.getAttribute('aria-invalid')).toBe('true');
    expect(descriptions).toEqual([
      INPUTS.hint,
      'The email address or the password is not correct.',
    ]);
  });
});
