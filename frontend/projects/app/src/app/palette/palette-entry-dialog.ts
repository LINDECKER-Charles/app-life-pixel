import { ChangeDetectionStrategy, Component, inject } from '@angular/core';
import { IonModal } from '@ionic/angular';
import { TranslocoPipe } from '@jsverse/transloco';
import { ModalLabel } from '../library/save/modal-label';
import { PaletteEntryFlow } from './palette-entry-flow';
import { PaletteEntryForm } from './palette-entry-form';

/**
 * The add/edit dialog, open while `PaletteEntryFlow` holds a mode, in the shared dialog anatomy
 * (`lp-dialog`); its form starts afresh.
 */
@Component({
  selector: 'lp-palette-entry-dialog',
  imports: [IonModal, ModalLabel, PaletteEntryForm, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <ion-modal
      class="lp-modal"
      [isOpen]="flow.current() !== null"
      [attr.aria-label]="headingKey() | transloco"
      [lpModalLabel]="headingKey() | transloco"
      (didDismiss)="flow.close()"
    >
      <ng-template>
        <div class="lp-dialog">
          <header class="lp-dialog__header">
            <h2 class="lp-dialog__title">{{ headingKey() | transloco }}</h2>
          </header>
          @if (flow.current(); as mode) {
            <lp-palette-entry-form [mode]="mode" />
          }
        </div>
      </ng-template>
    </ion-modal>
  `,
})
export class PaletteEntryDialog {
  protected readonly flow = inject(PaletteEntryFlow);

  protected headingKey(): string {
    return this.flow.current()?.kind === 'edit'
      ? 'palette.dialog.edit_title'
      : 'palette.dialog.add_title';
  }
}
