import { ChangeDetectionStrategy, Component, DestroyRef, inject } from '@angular/core';
import { IonButton } from '@ionic/angular';
import { TranslocoPipe } from '@jsverse/transloco';
import { Shortcuts } from '../editor/shortcuts';
import { ExportDialog } from './export-dialog';
import { ExportFlow } from './export-flow';

/**
 * The editor page's export button: opens the export dialog, also on Ctrl/⌘ E (editor.md, U5). It
 * takes no input, filling U1's stub in place; the dialog it opens lives in its own template so
 * that the editor page never changes.
 */
@Component({
  selector: 'lp-export-button',
  imports: [ExportDialog, IonButton, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <ion-button fill="outline" (click)="flow.open()">
      {{ 'export.action' | transloco }}
    </ion-button>
    <lp-export-dialog />
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
