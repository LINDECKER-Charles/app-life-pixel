import { inject, Injectable, signal } from '@angular/core';
import { EditorStore } from '../../editor/editor-store';
import { EngineStore } from '../../engine/engine-store';
import { isFileTooLarge, type ImportTooLarge } from './import-size';
import type { SpriteSheetValues } from './import-sprite-sheet-values';

/**
 * "Import sprite sheet": picks a PNG, asks the cell size and the frames' duration, then applies
 * `importSpriteSheet` after the active frame (editor.md, U4). Oversized files are refused before
 * they are read.
 */
@Injectable({ providedIn: 'root' })
export class ImportSpriteSheetFlow {
  private readonly editor = inject(EditorStore);
  private readonly engine = inject(EngineStore);
  private readonly pngSignal = signal<Uint8Array | null>(null);
  private readonly errorSignal = signal<ImportTooLarge | null>(null);

  readonly isOpen = signal(false);
  readonly error = this.errorSignal.asReadonly();

  async pickFile(file: File): Promise<void> {
    const limits = this.engine.limits();
    if (!limits) return;
    const tooLarge = isFileTooLarge(file, limits);
    this.errorSignal.set(tooLarge);
    if (tooLarge) return;
    this.pngSignal.set(new Uint8Array(await file.arrayBuffer()));
    this.isOpen.set(true);
  }

  async submit(values: SpriteSheetValues): Promise<void> {
    const png = this.pngSignal();
    const layer = this.editor.activeLayer();
    if (!png || layer === null) return;
    await this.engine.apply({
      kind: 'importSpriteSheet',
      layer,
      position: this.positionAfterActiveFrame(),
      png,
      cellWidth: values.cellWidth,
      cellHeight: values.cellHeight,
      durationMs: values.durationMs,
    });
    this.close();
  }

  close(): void {
    this.isOpen.set(false);
    this.pngSignal.set(null);
  }

  private positionAfterActiveFrame(): number {
    const frames = this.engine.document()?.frames ?? [];
    const index = frames.findIndex((frame) => frame.id === this.editor.activeFrame());
    return index < 0 ? frames.length : index + 1;
  }
}
