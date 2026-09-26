import { ChangeDetectionStrategy, Component, DestroyRef, inject } from '@angular/core';
import { TranslocoPipe } from '@jsverse/transloco';
import { Shortcuts } from '../editor/shortcuts';
import { Icon } from '../ui/icon/icon';
import { ExportDialog } from './export-dialog';
import { ExportFlow } from './export-flow';

/**
 * The document bar's export button, its primary action: opens the export dialog, also on Ctrl/⌘ E
 * (editor.md, U5). It takes no input; the dialog it opens lives in its own template so that the
 * editor page never changes.
 */
@Component({
  selector: 'lp-export-button',
  imports: [ExportDialog, Icon, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <button type="button" class="lp-button lp-button--primary" (click)="flow.open()">
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
  protected readonly flow = inject(ExportFlow);

  constructor() {
    const shortcuts = inject(Shortcuts);
    const unregister = shortcuts.register([
      { key: 'e', primary: true, label: 'export.action', action: () => void this.flow.open() },
    ]);
    inject(DestroyRef).onDestroy(unregister);
  }
}
