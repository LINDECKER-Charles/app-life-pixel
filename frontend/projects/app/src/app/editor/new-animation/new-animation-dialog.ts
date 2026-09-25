import { ChangeDetectionStrategy, Component, inject } from '@angular/core';
import { IonModal } from '@ionic/angular';
import { TranslocoPipe } from '@jsverse/transloco';
import { EngineStore } from '../../engine/engine-store';
import { NewAnimationFlow } from './new-animation-flow';
import { NewAnimationForm } from './new-animation-form';

/** The new-animation dialog, open while `NewAnimationFlow` says so; its form starts afresh. */
@Component({
  selector: 'lp-new-animation-dialog',
  imports: [IonModal, NewAnimationForm, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <ion-modal
      [isOpen]="flow.isOpen()"
      [attr.aria-label]="'editor.new.title' | transloco"
      (didDismiss)="flow.close()"
    >
      <ng-template>
        <div class="dialog">
          <h2 class="heading">{{ 'editor.new.title' | transloco }}</h2>
          @if (limits(); as limits) {
            <lp-new-animation-form [limits]="limits" />
          }
        </div>
      </ng-template>
    </ion-modal>
  `,
  styles: `
    .dialog {
      padding: var(--lp-space-5);
      overflow-y: auto;
      background: var(--lp-color-background);
    }
    .heading {
      margin: 0 0 var(--lp-space-4);
      font-size: var(--lp-font-size-large);
    }
  `,
})
export class NewAnimationDialog {
  protected readonly flow = inject(NewAnimationFlow);
  protected readonly limits = inject(EngineStore).limits;
}
