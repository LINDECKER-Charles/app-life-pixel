import { inject, Injectable, signal } from '@angular/core';
import { Router } from '@angular/router';
import { TranslocoService } from '@jsverse/transloco';
import { EngineStore } from '../../engine/engine-store';
import { CurrentAnimation } from '../../library/current-animation';
import { DiscardConfirmation } from './discard-confirmation';
import type { NewAnimationValues } from './new-animation-values';

const EDITOR_PATH = '/editor';

/**
 * Starting a new animation: the dialog's state, and the creation with a translated layer name.
 * The new animation is unsaved work (H8): the route leaves any saved animation's id.
 */
@Injectable({ providedIn: 'root' })
export class NewAnimationFlow {
  private readonly engine = inject(EngineStore);
  private readonly transloco = inject(TranslocoService);
  private readonly confirmation = inject(DiscardConfirmation);
  private readonly current = inject(CurrentAnimation);
  private readonly router = inject(Router);
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
    const before = this.engine.document();
    await this.engine.create({ ...values, layerName });
    const created = this.engine.document();
    if (created === null) return;
    this.open.set(false);
    if (created === before) return;
    this.current.setUnsaved();
    if (this.router.url.startsWith(`${EDITOR_PATH}/`)) await this.router.navigateByUrl(EDITOR_PATH);
  }

  close(): void {
    this.open.set(false);
  }
}
