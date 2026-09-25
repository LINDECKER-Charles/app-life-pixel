import { inject, Injectable, signal } from '@angular/core';
import { TranslocoService } from '@jsverse/transloco';
import { EngineStore } from '../../engine/engine-store';
import { DiscardConfirmation } from './discard-confirmation';
import type { NewAnimationValues } from './new-animation-values';

/** Starting a new animation: the dialog's state, and the creation with a translated layer name. */
@Injectable({ providedIn: 'root' })
export class NewAnimationFlow {
  private readonly engine = inject(EngineStore);
  private readonly transloco = inject(TranslocoService);
  private readonly confirmation = inject(DiscardConfirmation);
  private readonly open = signal(false);

  readonly isOpen = this.open.asReadonly();

  /** Opens the dialog; with unsaved work, only once the person agrees to lose it. */
  async start(): Promise<void> {
    if (this.engine.hasUnsavedWork() && !(await this.confirmation.confirm())) return;
    this.open.set(true);
  }

  /** Creates the animation, then closes the dialog once a document is open. */
  async create(values: NewAnimationValues): Promise<void> {
    const layerName = this.transloco.translate('editor.new.layer_name');
    await this.engine.create({ ...values, layerName });
    if (this.engine.document() !== null) this.open.set(false);
  }

  close(): void {
    this.open.set(false);
  }
}
