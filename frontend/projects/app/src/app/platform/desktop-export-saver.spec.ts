import { TestBed } from '@angular/core/testing';
import { mockIPC } from '@tauri-apps/api/mocks';
import type { ExportedFile } from '../engine/engine-types';
import { encodeBase64 } from './base64';
import { DesktopExportSaver } from './desktop-export-saver';
import { clearTauriMocks } from './testing/clear-tauri-mocks';

/** The bytes of an ASCII text, in this realm's `Uint8Array` (`TextEncoder`'s is a different one). */
function bytes(text: string): Uint8Array {
  return Uint8Array.from(text, (character) => character.charCodeAt(0));
}

const WASM: ExportedFile = {
  name: 'mascot.wasm',
  mediaType: 'application/wasm',
  bytes: bytes('binary'),
};
const LOADER: ExportedFile = {
  name: 'life-pixel.js',
  mediaType: 'text/javascript',
  bytes: bytes('loader'),
};

describe('DesktopExportSaver', () => {
  afterEach(() => clearTauriMocks());

  it('sends every file base64-encoded to export_save_files', async () => {
    const calls: unknown[] = [];
    mockIPC((command, payload) => {
      calls.push({ command, payload });
      return ['/exports/mascot.wasm', '/exports/life-pixel.js'];
    });
    TestBed.configureTestingModule({ providers: [DesktopExportSaver] });
    const saver = TestBed.inject(DesktopExportSaver);

    await saver.save([WASM, LOADER]);

    expect(calls).toEqual([
      {
        command: 'export_save_files',
        payload: {
          files: [
            { name: 'mascot.wasm', bytes: encodeBase64(WASM.bytes) },
            { name: 'life-pixel.js', bytes: encodeBase64(LOADER.bytes) },
          ],
        },
      },
    ]);
  });

  it('rejects with the command’s error when the write fails', async () => {
    mockIPC(() => {
      throw { code: 'export.write_failed', params: { path: '/exports/mascot.wasm' } };
    });
    TestBed.configureTestingModule({ providers: [DesktopExportSaver] });
    const saver = TestBed.inject(DesktopExportSaver);

    await expect(saver.save([WASM])).rejects.toEqual({
      code: 'export.write_failed',
      params: { path: '/exports/mascot.wasm' },
    });
  });

  it('resolves even when the person cancels the dialog', async () => {
    mockIPC(() => null);
    TestBed.configureTestingModule({ providers: [DesktopExportSaver] });
    const saver = TestBed.inject(DesktopExportSaver);

    await expect(saver.save([WASM])).resolves.toBeUndefined();
  });
});
