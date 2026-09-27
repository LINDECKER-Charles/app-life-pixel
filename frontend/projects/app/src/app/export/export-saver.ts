import { InjectionToken } from '@angular/core';
import type { ExportedFile } from '../engine/engine-types';

/**
 * Saves an export's files somewhere. The web default downloads each from a `Blob`; T2 gives the
 * desktop one, through the system's save dialog (editor.md, U5).
 */
export interface ExportSaver {
  save(files: readonly ExportedFile[]): Promise<void>;
}

function downloadFile(file: ExportedFile): void {
  const url = URL.createObjectURL(new Blob([new Uint8Array(file.bytes)], { type: file.mediaType }));
  const link = document.createElement('a');
  link.href = url;
  link.download = file.name;
  link.click();
  URL.revokeObjectURL(url);
}

class WebExportSaver implements ExportSaver {
  async save(files: readonly ExportedFile[]): Promise<void> {
    for (const file of files) downloadFile(file);
  }
}

export const EXPORT_SAVER = new InjectionToken<ExportSaver>('EXPORT_SAVER', {
  providedIn: 'root',
  factory: () => new WebExportSaver(),
});
