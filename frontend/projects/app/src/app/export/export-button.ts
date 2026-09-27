import { ChangeDetectionStrategy, Component, computed, DestroyRef, inject } from '@angular/core';
import { TranslocoPipe } from '@jsverse/transloco';
import { Shortcuts } from '../editor/shortcuts';
import { EngineStore } from '../engine/engine-store';
import { Icon } from '../ui/icon/icon';
import { ExportDialog } from './export-dialog';
import { ExportFlow } from './export-flow';

/**
 * The document bar's export button, its primary action: opens the export dialog, also on Ctrl/⌘ E
 * (editor.md, U5). It takes no input; the dialog it opens lives in its own template so that the
 * editor page never changes. Like Save, it is disabled while there is no animation to export.
 */
@Component({
  selector: 'lp-export-button',
  imports: [ExportDialog, Icon, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <button
      type="button"
      class="lp-button lp-button--primary"
      [disabled]="!hasDocument()"
      (click)="open()"
    >
      <lp-icon name="export" />
      {{ 'export.action' | transloco }}
    </button>
    <lp-export-dialog />
  `,
  styles: `
    :host {
      display: inline-flex;
    }
  `,
})
export class ExportButton {
  private readonly flow = inject(ExportFlow);
  private readonly engine = inject(EngineStore);

  protected readonly hasDocument = computed(() => this.engine.document() !== null);

  constructor() {
    const shortcuts = inject(Shortcuts);
    const unregister = shortcuts.register([
      { key: 'e', primary: true, label: 'export.action', action: () => this.open() },
    ]);
    inject(DestroyRef).onDestroy(unregister);
  }

  protected open(): void {
    if (this.hasDocument()) void this.flow.open();
  }
}
