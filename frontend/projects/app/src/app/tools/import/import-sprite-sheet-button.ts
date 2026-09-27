import { ChangeDetectionStrategy, Component, inject } from '@angular/core';
import { TranslocoPipe } from '@jsverse/transloco';
import { EngineStore } from '../../engine/engine-store';
import { Icon } from '../../ui/icon/icon';
import { Tooltip } from '../../ui/tooltip/tooltip';
import { ImportSpriteSheetDialog } from './import-sprite-sheet-dialog';
import { ImportSpriteSheetFlow } from './import-sprite-sheet-flow';

/** "Import sprite sheet": a hidden file input behind a button, the dialog, and the size error. */
@Component({
  selector: 'lp-import-sprite-sheet-button',
  imports: [Icon, ImportSpriteSheetDialog, Tooltip, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <button
      type="button"
      class="lp-icon-button"
      [attr.aria-label]="'tools.import_sprite_sheet' | transloco"
      [disabled]="!engine.document()"
      [lpTooltip]="'tools.import_sprite_sheet' | transloco"
      (click)="fileInput.click()"
    >
      <lp-icon name="import-sheet" size="large" />
    </button>
    <input
      #fileInput
      class="lp-visually-hidden"
      type="file"
      accept="image/png"
      tabindex="-1"
      [attr.aria-label]="'tools.import_sprite_sheet' | transloco"
      (change)="onChange(fileInput)"
    />
    @if (flow.error(); as error) {
      <p class="hint" role="alert">{{ 'errors.import.image_too_large' | transloco: error }}</p>
    }
    <lp-import-sprite-sheet-dialog />
  `,
  styles: `
    /* The button is one cell of the tool rail's grid; an error spans the whole row under it. */
    :host {
      display: contents;
    }
    .hint {
      grid-column: 1 / -1;
      margin: var(--lp-space-1) 0 0;
      font-size: var(--lp-font-size-small);
      color: var(--lp-color-danger);
    }
  `,
})
export class ImportSpriteSheetButton {
  protected readonly engine = inject(EngineStore);
  protected readonly flow = inject(ImportSpriteSheetFlow);

  protected async onChange(input: HTMLInputElement): Promise<void> {
    const file = input.files?.[0];
    input.value = '';
    if (file) await this.flow.pickFile(file);
  }
}
