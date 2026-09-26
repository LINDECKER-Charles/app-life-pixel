import { Injectable } from '@angular/core';
import { invoke } from '@tauri-apps/api/core';
import type { ExportedFile } from '../engine/engine-types';
import type { ExportSaver } from '../export/export-saver';
import { encodeBase64 } from './base64';

/**
 * The desktop's `ExportSaver` (editor.md, U5): T1's `export_save_files`, through the system's
 * save or folder dialog — never a download.
 */
@Injectable()
export class DesktopExportSaver implements ExportSaver {
  async save(files: readonly ExportedFile[]): Promise<void> {
    await invoke('export_save_files', {
      files: files.map((file) => ({ name: file.name, bytes: encodeBase64(file.bytes) })),
    });
  }
}
