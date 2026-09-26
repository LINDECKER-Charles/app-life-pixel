import { TestBed } from '@angular/core/testing';
import { mockIPC } from '@tauri-apps/api/mocks';
import { TranslocoService } from '@jsverse/transloco';
import { of } from 'rxjs';
import { desktopPreferredLanguage, DesktopPreferencesStore } from './desktop-preferences';
import { clearTauriMocks } from './testing/clear-tauri-mocks';

const SETTINGS = {
  libraryPath: '/home/pixel/Life Pixel',
  language: 'fr',
  theme: 'dark',
  motion: 'reduce',
};

function setUp(handler: (command: string, payload: unknown) => unknown): {
  store: DesktopPreferencesStore;
  transloco: { setActiveLang: ReturnType<typeof vi.fn>; load: ReturnType<typeof vi.fn> };
} {
  mockIPC((command, payload) => handler(command, payload));
  const transloco = {
    langChanges$: of('en'),
    getActiveLang: () => 'en',
    setActiveLang: vi.fn(),
    load: vi.fn().mockReturnValue(of(undefined)),
  };
  TestBed.configureTestingModule({
    providers: [DesktopPreferencesStore, { provide: TranslocoService, useValue: transloco }],
  });
  return { store: TestBed.inject(DesktopPreferencesStore), transloco };
}

describe('DesktopPreferencesStore', () => {
  afterEach(() => clearTauriMocks());

  it('reads settings_get at start-up and exposes theme, motion and the library folder', async () => {
    const { store } = setUp(() => SETTINGS);

    await vi.waitFor(() => expect(store.theme()).toBe('dark'));
    expect(store.motion()).toBe('reduce');
    expect(store.libraryPath()).toBe('/home/pixel/Life Pixel');
  });

  it('changes the theme, merging it into the settings already read', async () => {
    const calls: unknown[] = [];
    const { store } = setUp((command, payload) => {
      calls.push({ command, payload });
      return command === 'settings_set' ? { ...SETTINGS, theme: 'light' } : SETTINGS;
    });
    await vi.waitFor(() => expect(store.theme()).toBe('dark'));

    await store.setTheme('light');

    expect(store.theme()).toBe('light');
    expect(calls).toContainEqual({
      command: 'settings_set',
      payload: { settings: { ...SETTINGS, theme: 'light' } },
    });
  });

  it('changes the motion the same way', async () => {
    const { store } = setUp((command) =>
      command === 'settings_set' ? { ...SETTINGS, motion: 'system' } : SETTINGS,
    );
    await vi.waitFor(() => expect(store.motion()).toBe('reduce'));

    await store.setMotion('system');

    expect(store.motion()).toBe('system');
  });

  it('loads the language’s catalogue, activates it, then persists it', async () => {
    const { store, transloco } = setUp((command) =>
      command === 'settings_set' ? { ...SETTINGS, language: 'de' } : SETTINGS,
    );
    await vi.waitFor(() => expect(store.libraryPath()).toBe('/home/pixel/Life Pixel'));

    await store.setLanguage('de');

    expect(transloco.load).toHaveBeenCalledWith('de');
    expect(transloco.setActiveLang).toHaveBeenCalledWith('de');
  });

  it('persists the folder chosen in the system’s dialog', async () => {
    const calls: unknown[] = [];
    const { store } = setUp((command, payload) => {
      calls.push({ command, payload });
      return command === 'settings_set' ? { ...SETTINGS, libraryPath: '/new/library' } : SETTINGS;
    });
    await vi.waitFor(() => expect(store.libraryPath()).toBe('/home/pixel/Life Pixel'));

    await store.setLibraryFolder('/new/library');

    expect(store.libraryPath()).toBe('/new/library');
    expect(calls).toContainEqual({
      command: 'settings_set',
      payload: { settings: { ...SETTINGS, libraryPath: '/new/library' } },
    });
  });

  it('rejects when the settings cannot be changed', async () => {
    const { store } = setUp((command) => {
      if (command === 'settings_get') return SETTINGS;
      throw { code: 'request.malformed', params: {} };
    });
    await vi.waitFor(() => expect(store.theme()).toBe('dark'));

    await expect(store.setTheme('light')).rejects.toEqual({
      code: 'request.malformed',
      params: {},
    });
  });
});

describe('desktopPreferredLanguage', () => {
  afterEach(() => clearTauriMocks());

  it('answers the language kept in the desktop settings', async () => {
    mockIPC(() => SETTINGS);
    await expect(desktopPreferredLanguage()).resolves.toBe('fr');
  });

  it('answers undefined when no language was chosen yet', async () => {
    mockIPC(() => ({ ...SETTINGS, language: null }));
    await expect(desktopPreferredLanguage()).resolves.toBeUndefined();
  });
});
