import { inject, Injectable, signal } from '@angular/core';
import { EngineStore } from '../engine/engine-store';
import type { Color } from '../engine/engine-types';

export type PaletteEntryMode =
  | { readonly kind: 'add' }
  | { readonly kind: 'edit'; readonly index: number; readonly color: Color };

/** Adding or editing one palette entry: the dialog's state, and its submission (editor.md, U4). */
@Injectable({ providedIn: 'root' })
export class PaletteEntryFlow {
  private readonly engine = inject(EngineStore);
  private readonly modeSignal = signal<PaletteEntryMode | null>(null);

  readonly current = this.modeSignal.asReadonly();

  openAdd(): void {
    this.modeSignal.set({ kind: 'add' });
  }

  openEdit(index: number, color: Color): void {
    this.modeSignal.set({ kind: 'edit', index, color });
  }

  close(): void {
    this.modeSignal.set(null);
  }

  /** Applies the entry, then closes the dialog once the palette reflects it. */
  async submit(color: Color): Promise<void> {
    const mode = this.modeSignal();
    if (!mode) return;
    if (mode.kind === 'add') {
      const sizeBefore = this.engine.document()?.palette.length ?? 0;
      await this.engine.apply({ kind: 'addPaletteEntry', color });
      if ((this.engine.document()?.palette.length ?? 0) > sizeBefore) this.close();
    } else {
      await this.engine.apply({ kind: 'setPaletteEntry', index: mode.index, color });
      if (this.engine.document()?.palette[mode.index] === color) this.close();
    }
  }
}
