import {
  ChangeDetectionStrategy,
  Component,
  importProvidersFrom,
  signal,
  viewChild,
} from '@angular/core';
import { TestBed } from '@angular/core/testing';
import { TranslocoService, TranslocoTestingModule } from '@jsverse/transloco';
import { firstValueFrom } from 'rxjs';
import en from '../../../../../../../i18n/en.json';
import { ErrorSummary } from './error-summary';
import type { FormErrorEntry } from './form-errors';

const I18N_TESTING = { langs: { en }, translocoConfig: { availableLangs: ['en'] } };

const EMAIL_REQUIRED = { field: 'email', error: { key: 'auth.form.email_required', params: {} } };
const PASSWORD_REQUIRED = {
  field: 'password',
  error: { key: 'auth.form.password_required', params: {} },
};
const CSRF = { field: 'form', error: { key: 'errors.auth.csrf', params: {} } };

@Component({
  imports: [ErrorSummary],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <form>
      <lp-error-summary [entries]="entries()" [targets]="targets" />
      <input id="test-email" />
      <input id="test-password" />
    </form>
  `,
})
class Host {
  readonly entries = signal<readonly FormErrorEntry[]>([]);
  readonly targets = { email: 'test-email', password: 'test-password' };
  readonly summary = viewChild.required(ErrorSummary);
}

async function render(): Promise<{
  host: Host;
  element: HTMLElement;
  settle: () => Promise<void>;
}> {
  TestBed.configureTestingModule({
    providers: [importProvidersFrom(TranslocoTestingModule.forRoot(I18N_TESTING))],
  });
  await firstValueFrom(TestBed.inject(TranslocoService).load('en'));
  const fixture = TestBed.createComponent(Host);
  await fixture.whenStable();
  return {
    host: fixture.componentInstance,
    element: fixture.nativeElement,
    settle: () => fixture.whenStable(),
  };
}

describe('ErrorSummary', () => {
  it('shows nothing while the form has no error', async () => {
    const { element } = await render();

    expect(element.querySelector('[role="alert"]')).toBeNull();
  });

  it('lists each error as an alert, a field error linking to its field', async () => {
    const { host, element, settle } = await render();

    host.entries.set([EMAIL_REQUIRED, CSRF]);
    await settle();

    const alert = element.querySelector('[role="alert"]') as HTMLElement;
    const items = [...alert.querySelectorAll('li')];
    expect(items.map((item) => item.textContent?.trim())).toEqual([
      'Enter your email address.',
      'This request could not be verified. Reload the page and try again.',
    ]);
    expect(items[0].querySelector('a')?.getAttribute('href')).toBe('#test-email');
    expect(items[1].querySelector('a')).toBeNull();
  });

  it('moves the focus to the first invalid field', async () => {
    const { host, element, settle } = await render();

    host.entries.set([PASSWORD_REQUIRED, EMAIL_REQUIRED]);
    host.summary().focusFirstError();
    await settle();

    expect(document.activeElement).toBe(element.querySelector('#test-password'));
  });

  it('moves the focus to itself when only the form as a whole was refused', async () => {
    const { host, element, settle } = await render();

    host.entries.set([CSRF]);
    host.summary().focusFirstError();
    await settle();

    expect(document.activeElement).toBe(element.querySelector('lp-status-banner'));
  });

  it('focuses a field from its link', async () => {
    const { host, element, settle } = await render();
    host.entries.set([EMAIL_REQUIRED]);
    await settle();

    (element.querySelector('a') as HTMLAnchorElement).click();

    expect(document.activeElement).toBe(element.querySelector('#test-email'));
  });
});
