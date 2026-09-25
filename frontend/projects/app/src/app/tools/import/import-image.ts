import { inject, Injectable, signal } from '@angular/core';
import { EditorStore } from '../../editor/editor-store';
import { EngineStore } from '../../engine/engine-store';
import { isFileTooLarge, type ImportTooLarge } from './import-size';

/**
 * "Import image": applies `importImage` on the active layer and frame, at the top-left corner
 * (editor.md, U4). Oversized files are refused before they are read.
 */
@Injectable({ providedIn: 'root' })
export class ImportImage {
  private readonly editor = inject(EditorStore);
  private readonly engine = inject(EngineStore);
  private readonly errorSignal = signal<ImportTooLarge | null>(null);

  readonly error = this.errorSignal.asReadonly();

  async importFile(file: File): Promise<void> {
    const limits = this.engine.limits();
    const layer = this.editor.activeLayer();
    const frame = this.editor.activeFrame();
    if (!limits || layer === null || frame === null) return;
    const tooLarge = isFileTooLarge(file, limits);
    this.errorSignal.set(tooLarge);
    if (tooLarge) return;
    const png = new Uint8Array(await file.arrayBuffer());
    await this.engine.apply({ kind: 'importImage', layer, frame, png, at: { x: 0, y: 0 } });
  }
}
