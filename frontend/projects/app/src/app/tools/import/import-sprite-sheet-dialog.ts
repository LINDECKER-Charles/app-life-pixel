import { ChangeDetectionStrategy, Component, inject } from '@angular/core';
import { IonModal } from '@ionic/angular';
import { TranslocoPipe } from '@jsverse/transloco';
import { EngineStore } from '../../engine/engine-store';
import { ImportSpriteSheetFlow } from './import-sprite-sheet-flow';
import { ImportSpriteSheetForm } from './import-sprite-sheet-form';

/** The sprite-sheet dialog, open while `ImportSpriteSheetFlow` holds a picked file. */
@Component({
  selector: 'lp-import-sprite-sheet-dialog',
  imports: [IonModal, ImportSpriteSheetForm, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <ion-modal
      [isOpen]="flow.isOpen()"
      [attr.aria-label]="'tools.sprite_sheet.title' | transloco"
      (didDismiss)="flow.close()"
    >
      <ng-template>
        <div class="dialog">
          <h2 class="heading">{{ 'tools.sprite_sheet.title' | transloco }}</h2>
          @if (limits(); as limits) {
            <lp-import-sprite-sheet-form [limits]="limits" />
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
export class ImportSpriteSheetDialog {
  protected readonly flow = inject(ImportSpriteSheetFlow);
  protected readonly limits = inject(EngineStore).limits;
}
