import { ChangeDetectionStrategy, Component, computed, DestroyRef, inject } from '@angular/core';
import { TranslocoPipe } from '@jsverse/transloco';
import { Shortcuts } from '../../editor/shortcuts';
import { EngineStore } from '../../engine/engine-store';
import { Icon } from '../../ui/icon/icon';
import { SaveDialog } from './save-dialog';
import { SaveFlow } from './save-flow';

/**
 * The document bar's Save button, also on Ctrl/⌘ S (accounts.md, H8). It stays focusable while a
 * save runs, marked busy — `SaveFlow` ignores a second save meanwhile —; the save state beside the
 * title (`lp-save-state-pill`) says how saving went. The dialogs saving asks through live in its
 * own template.
 */
@Component({
  selector: 'lp-save-button',
  imports: [Icon, SaveDialog, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <button
      type="button"
      class="lp-button lp-button--secondary"
      [disabled]="!hasDocument()"
      [attr.aria-busy]="isSaving() ? 'true' : null"
      (click)="save()"
    >
      <lp-icon name="save" />
      {{ 'library.save.action' | transloco }}
    </button>
    <lp-save-dialog />
  `,
  styles: `
    :host {
      display: inline-flex;
    }
  `,
})
export class SaveButton {
  private readonly flow = inject(SaveFlow);
  private readonly engine = inject(EngineStore);

  protected readonly hasDocument = computed(() => this.engine.document() !== null);
  protected readonly isSaving = computed(() => this.flow.status() === 'saving');

  constructor() {
    const unregister = inject(Shortcuts).register([
      { key: 's', primary: true, label: 'library.save.action', action: () => this.save() },
    ]);
    inject(DestroyRef).onDestroy(unregister);
  }

  protected save(): void {
    void this.flow.save();
  }
}
