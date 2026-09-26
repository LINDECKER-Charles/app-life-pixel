import { mockIPC } from '@tauri-apps/api/mocks';
import { TestBed } from '@angular/core/testing';
import { isDesktop, PlatformService, type PlatformInfo } from './platform';
import { clearTauriMocks } from './testing/clear-tauri-mocks';

const INFO: PlatformInfo = {
  version: '0.0.0',
  os: 'macos',
  libraryPath: '/home/pixel/Life Pixel',
  cliPath: null,
};

describe('isDesktop', () => {
  afterEach(() => clearTauriMocks());

  it('is false without Tauri’s internals', () => {
    expect(isDesktop()).toBe(false);
  });

  it('is true once Tauri’s internals exist', () => {
    mockIPC(() => null);
    expect(isDesktop()).toBe(true);
  });
});

describe('PlatformService', () => {
  afterEach(() => clearTauriMocks());

  it('reads the platform info through platform_info', async () => {
    mockIPC((command) => (command === 'platform_info' ? INFO : null));
    const service = TestBed.inject(PlatformService);

    await expect(service.info()).resolves.toEqual(INFO);
  });

  it('rejects with the command’s error', async () => {
    mockIPC(() => {
      throw { code: 'service.unavailable', params: {} };
    });
    const service = TestBed.inject(PlatformService);

    await expect(service.info()).rejects.toEqual({ code: 'service.unavailable', params: {} });
  });

  it('picks the library folder, or answers null when the person cancels', async () => {
    mockIPC((command) =>
      command === 'settings_pick_library_folder' ? '/home/pixel/New Library' : null,
    );
    const service = TestBed.inject(PlatformService);

    await expect(service.pickLibraryFolder()).resolves.toBe('/home/pixel/New Library');
  });

  it('answers null when the folder dialog is cancelled', async () => {
    mockIPC(() => null);
    const service = TestBed.inject(PlatformService);

    await expect(service.pickLibraryFolder()).resolves.toBeNull();
  });
});
