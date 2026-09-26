import { HttpTestingController } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';
import { openAccountPage, waitForEffects } from './testing/account-test-support';
import { PasswordField } from './password-field';

function nativeInput(fixture: { nativeElement: HTMLElement }): HTMLInputElement | null | undefined {
  return fixture.nativeElement.querySelector('ion-input input');
}

describe('PasswordField', () => {
  afterEach(() => TestBed.inject(HttpTestingController).verify());

  it('hides the password by default, and shows it once toggled', async () => {
    const fixture = await openAccountPage(PasswordField, null, { label: 'Password' });

    await waitForEffects(() => expect(nativeInput(fixture)?.getAttribute('type')).toBe('password'));

    fixture.nativeElement.querySelector('button')?.dispatchEvent(new Event('click'));

    await waitForEffects(() => expect(nativeInput(fixture)?.getAttribute('type')).toBe('text'));
  });

  it('emits the value typed', async () => {
    const fixture = await openAccountPage(PasswordField, null, { label: 'Password' });

    let emitted = '';
    fixture.componentInstance.valueChange.subscribe((value) => (emitted = value));
    fixture.nativeElement
      .querySelector('ion-input')
      ?.dispatchEvent(new CustomEvent('ionInput', { detail: { value: 'a secret' } }));

    expect(emitted).toBe('a secret');
  });
});
