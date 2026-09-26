import { ChangeDetectionStrategy, Component, inject } from '@angular/core';
import { IonModal } from '@ionic/angular';
import { TranslocoPipe } from '@jsverse/transloco';
import { EngineStore } from '../../engine/engine-store';
import { Icon } from '../../ui/icon/icon';
import { ImportSpriteSheetFlow } from './import-sprite-sheet-flow';
import { ImportSpriteSheetForm } from './import-sprite-sheet-form';

/**
 * The sprite-sheet dialog, open while `ImportSpriteSheetFlow` holds a picked file: what the import
 * does, then its form, whose body scrolls above the actions (`_dialog.scss`).
 */
@Component({
  selector: 'lp-import-sprite-sheet-dialog',
  imports: [Icon, IonModal, ImportSpriteSheetForm, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <ion-modal
      class="lp-modal"
      [isOpen]="flow.isOpen()"
      [attr.aria-label]="'tools.sprite_sheet.title' | transloco"
      (didPresent)="focusFirstField($event)"
      (didDismiss)="flow.close()"
    >
      <ng-template>
        <div class="lp-dialog">
          <div class="lp-dialog__header">
            <h2 class="lp-dialog__title">{{ 'tools.sprite_sheet.title' | transloco }}</h2>
            <button
              type="button"
              class="lp-icon-button"
              [attr.aria-label]="'common.close' | transloco"
              (click)="flow.close()"
            >
              <lp-icon name="close" />
            </button>
          </div>
          <p class="lp-dialog__context">{{ 'import.sheet.context' | transloco }}</p>
          @if (limits(); as limits) {
            <lp-import-sprite-sheet-form [limits]="limits" />
          }
        </div>
      </ng-template>
    </ion-modal>
  `,
})
export class ImportSpriteSheetDialog {
  protected readonly flow = inject(ImportSpriteSheetFlow);
  protected readonly limits = inject(EngineStore).limits;

  /** An entry dialog starts on its first field, which Ionic then leaves focused. */
  protected focusFirstField(event: Event): void {
    if (event.target instanceof HTMLElement) event.target.querySelector('input')?.focus();
  }
}
