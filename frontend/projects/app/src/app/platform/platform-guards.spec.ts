import { TestBed } from '@angular/core/testing';
import { provideRouter, Router, UrlTree } from '@angular/router';
import { mockIPC } from '@tauri-apps/api/mocks';
import { desktopOnly, hostedOnly } from './platform-guards';
import { clearTauriMocks } from './testing/clear-tauri-mocks';

function run(guard: typeof hostedOnly): boolean | UrlTree {
  TestBed.configureTestingModule({ providers: [provideRouter([])] });
  return TestBed.runInInjectionContext(() => guard({} as never, {} as never) as boolean | UrlTree);
}

describe('hostedOnly', () => {
  afterEach(() => clearTauriMocks());

  it('lets the web app through', () => {
    expect(run(hostedOnly)).toBe(true);
  });

  it('bounces the desktop app to the editor', () => {
    mockIPC(() => null);

    const result = run(hostedOnly);

    expect(result).toBeInstanceOf(UrlTree);
    expect(TestBed.inject(Router).serializeUrl(result as UrlTree)).toBe('/editor');
  });
});

describe('desktopOnly', () => {
  afterEach(() => clearTauriMocks());

  it('lets the desktop app through', () => {
    mockIPC(() => null);

    expect(run(desktopOnly)).toBe(true);
  });

  it('bounces the web app to the editor', () => {
    const result = run(desktopOnly);

    expect(result).toBeInstanceOf(UrlTree);
    expect(TestBed.inject(Router).serializeUrl(result as UrlTree)).toBe('/editor');
  });
});
