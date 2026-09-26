import { ChangeDetectionStrategy, Component, inject } from '@angular/core';
import { IonModal } from '@ionic/angular';
import { TranslocoPipe } from '@jsverse/transloco';
import { EngineStore } from '../../engine/engine-store';
import { NewAnimationFlow } from './new-animation-flow';
import { NewAnimationForm } from './new-animation-form';

/**
 * The new-animation dialog, open while `NewAnimationFlow` says so, in the shared dialog anatomy
 * (`lp-dialog`); its form starts afresh, and the title field takes the focus once it is shown.
 */
@Component({
  selector: 'lp-new-animation-dialog',
  imports: [IonModal, NewAnimationForm, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <ion-modal
      class="lp-modal"
      [isOpen]="flow.isOpen()"
      [attr.aria-label]="'editor.new.title' | transloco"
      (didPresent)="focusTitle($event)"
      (didDismiss)="flow.close()"
    >
      <ng-template>
        <div class="lp-dialog">
          <header class="lp-dialog__header">
            <h2 class="lp-dialog__title">{{ 'editor.new.title' | transloco }}</h2>
          </header>
          <p class="lp-dialog__context">{{ 'editor.new.context' | transloco }}</p>
          @if (limits(); as limits) {
            <lp-new-animation-form [limits]="limits" />
          }
        </div>
      </ng-template>
    </ion-modal>
  `,
})
export class NewAnimationDialog {
  protected readonly flow = inject(NewAnimationFlow);
  protected readonly limits = inject(EngineStore).limits;

  /**
   * Focuses the title — the form's first field —, its default selected so that typing replaces it
   * (journeys.md §1).
   */
  protected focusTitle(event: Event): void {
    const modal = event.target as HTMLElement;
    const field = modal.querySelector<HTMLInputElement>('lp-new-animation-form input');
    field?.focus();
    field?.select();
  }
}
