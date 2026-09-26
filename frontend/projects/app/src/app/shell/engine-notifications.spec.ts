import { importProvidersFrom, signal } from '@angular/core';
import { type ComponentFixture, TestBed } from '@angular/core/testing';
import { TranslocoService, TranslocoTestingModule } from '@jsverse/transloco';
import { firstValueFrom } from 'rxjs';
import en from '../../../../../../i18n/en.json';
import { type EngineNotification, EngineStore } from '../engine/engine-store';
import { ICON_PATHS } from '../ui/icon/icon-paths';
import { EngineNotifications } from './engine-notifications';

const I18N_TESTING = { langs: { en }, translocoConfig: { availableLangs: ['en'] } };

/** An `EngineStore` holding its notifications only, dismissed as the real one does. */
class FakeEngineStore {
  readonly notifications = signal<readonly EngineNotification[]>([
    { id: 1, key: 'errors.edit.layer_not_found', params: {} },
  ]);

  dismissNotification(id: number): void {
    this.notifications.update((list) => list.filter((notification) => notification.id !== id));
  }
}

async function render(): Promise<ComponentFixture<EngineNotifications>> {
  TestBed.configureTestingModule({
    providers: [
      importProvidersFrom(TranslocoTestingModule.forRoot(I18N_TESTING)),
      { provide: EngineStore, useClass: FakeEngineStore },
    ],
  });
  await firstValueFrom(TestBed.inject(TranslocoService).load('en'));
  const fixture = TestBed.createComponent(EngineNotifications);
  await fixture.whenStable();
  return fixture;
}

describe('EngineNotifications', () => {
  it('shows each refusal as an error: icon, the word and the translated message', async () => {
    const root: HTMLElement = (await render()).nativeElement;
    const notification = root.querySelector('li');

    expect(notification?.querySelector('path')?.getAttribute('d')).toBe(ICON_PATHS.error.outline);
    expect(notification?.textContent).toContain(en['common.status.error']);
    expect(notification?.textContent).toContain(en['errors.edit.layer_not_found']);
    expect(root.querySelector('ul')?.getAttribute('aria-live')).toBe('polite');
  });

  it('keeps a message until it is dismissed by its named button', async () => {
    const fixture = await render();
    const root: HTMLElement = fixture.nativeElement;
    const dismiss = root.querySelector<HTMLButtonElement>('li button');

    expect(dismiss?.getAttribute('aria-label')).toBe(en['common.dismiss']);
    dismiss?.click();
    await fixture.whenStable();

    expect(root.querySelector('li')).toBeNull();
  });
});
