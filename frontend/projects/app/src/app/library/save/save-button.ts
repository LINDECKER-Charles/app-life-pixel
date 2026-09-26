import { ChangeDetectionStrategy, Component, computed, DestroyRef, inject } from '@angular/core';
import { IonButton } from '@ionic/angular';
import { TranslocoPipe } from '@jsverse/transloco';
import { Shortcuts } from '../../editor/shortcuts';
import { EngineStore } from '../../engine/engine-store';
import { SaveDialog } from './save-dialog';
import { SaveFlow } from './save-flow';

/**
 * The editor header's Save button, also on Ctrl/⌘ S (accounts.md, H8), with a status saying
 * when the work is saved. The dialogs saving asks through live in its own template.
 */
@Component({
  selector: 'lp-save-button',
  imports: [IonButton, SaveDialog, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <ion-button
      fill="solid"
      [disabled]="flow.status() === 'saving' || !hasDocument()"
      (click)="save()"
    >
      {{ 'library.save.action' | transloco }}
    </ion-button>
    <span class="status" role="status">
      @if (flow.status() === 'saving') {
        {{ 'library.save.saving' | transloco }}
      } @else if (isSaved()) {
        {{ 'library.save.saved' | transloco }}
      }
    </span>
    <lp-save-dialog />
  `,
  styles: `
    :host {
      display: inline-flex;
      align-items: center;
      gap: var(--lp-space-2);
    }
    .status {
      font-size: var(--lp-font-size-small);
      color: var(--lp-color-text-muted);
    }
  `,
})
export class SaveButton {
  protected readonly flow = inject(SaveFlow);
  private readonly engine = inject(EngineStore);

  protected readonly hasDocument = computed(() => this.engine.document() !== null);
  protected readonly isSaved = computed(
    () => this.flow.status() === 'saved' && !this.engine.hasUnsavedWork(),
  );

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
