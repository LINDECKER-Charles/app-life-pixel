import { ChangeDetectionStrategy, Component, inject } from '@angular/core';
import { IonModal } from '@ionic/angular';
import { TranslocoPipe } from '@jsverse/transloco';
import { PaletteEntryFlow } from './palette-entry-flow';
import { PaletteEntryForm } from './palette-entry-form';

/** The add/edit dialog, open while `PaletteEntryFlow` holds a mode; its form starts afresh. */
@Component({
  selector: 'lp-palette-entry-dialog',
  imports: [IonModal, PaletteEntryForm, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <ion-modal
      [isOpen]="flow.current() !== null"
      [attr.aria-label]="headingKey() | transloco"
      (didDismiss)="flow.close()"
    >
      <ng-template>
        <div class="dialog">
          <h2 class="heading">{{ headingKey() | transloco }}</h2>
          @if (flow.current(); as mode) {
            <lp-palette-entry-form [mode]="mode" />
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
export class PaletteEntryDialog {
  protected readonly flow = inject(PaletteEntryFlow);

  protected headingKey(): string {
    return this.flow.current()?.kind === 'edit'
      ? 'palette.dialog.edit_title'
      : 'palette.dialog.add_title';
  }
}
