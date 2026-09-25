import { importProvidersFrom } from '@angular/core';
import { TestBed } from '@angular/core/testing';
import { provideIonicAngular } from '@ionic/angular';
import { TranslocoService, TranslocoTestingModule } from '@jsverse/transloco';
import { firstValueFrom } from 'rxjs';
import en from '../../../../../../i18n/en.json';
import { Shortcuts } from '../editor/shortcuts';
import { ShortcutsHelpDialog } from './shortcuts-help-dialog';
import { ShortcutsHelpState } from './shortcuts-help-state';

const I18N_TESTING = { langs: { en }, translocoConfig: { availableLangs: ['en'] } };

describe('ShortcutsHelpDialog', () => {
  async function configure(): Promise<void> {
    TestBed.configureTestingModule({
      providers: [
        provideIonicAngular({ animated: false }),
        importProvidersFrom(TranslocoTestingModule.forRoot(I18N_TESTING)),
      ],
    });
    await firstValueFrom(TestBed.inject(TranslocoService).load('en'));
  }

  afterEach(() => document.body.replaceChildren());

  it('opens and lists every registered shortcut once ShortcutsHelpState opens it', async () => {
    await configure();
    TestBed.inject(Shortcuts).register([
      { key: 'b', label: 'tools.tool.pencil', action: () => undefined },
    ]);
    TestBed.createComponent(ShortcutsHelpDialog);

    TestBed.inject(ShortcutsHelpState).open();

    const dialog = await vi.waitFor(() => {
      const rows = document.querySelectorAll('.row');
      if (rows.length === 0) throw new Error('the dialog is not open');
      return rows;
    });
    expect(dialog.length).toBeGreaterThanOrEqual(1);
    expect(document.body.textContent).toContain('B');
  });

  it('closes on the close button', async () => {
    await configure();
    TestBed.createComponent(ShortcutsHelpDialog);
    const state = TestBed.inject(ShortcutsHelpState);
    state.open();

    const button = await vi.waitFor(() => {
      const candidate = document.querySelector<HTMLButtonElement>('.secondary');
      if (!candidate) throw new Error('the dialog is not open');
      return candidate;
    });
    button.click();

    expect(state.isOpen()).toBe(false);
  });
});
