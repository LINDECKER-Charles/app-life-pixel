import { ChangeDetectionStrategy, Component, inject } from '@angular/core';
import { IonModal } from '@ionic/angular';
import { TranslocoPipe } from '@jsverse/transloco';
import { Shortcuts } from '../editor/shortcuts';
import { describeShortcut } from './shortcut-label';
import { ShortcutsHelpState } from './shortcuts-help-state';

/** Lists every registered shortcut (editor.md, U4), opened by the tool bar's `?` button. */
@Component({
  selector: 'lp-shortcuts-help-dialog',
  imports: [IonModal, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './shortcuts-help-dialog.html',
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
    .list {
      display: flex;
      flex-direction: column;
      gap: var(--lp-space-2);
      padding: 0;
      margin: 0 0 var(--lp-space-4);
      list-style: none;
    }
    .row {
      display: flex;
      justify-content: space-between;
      gap: var(--lp-space-4);
    }
    .keys {
      font-family: var(--lp-font-family-mono);
      color: var(--lp-color-text-muted);
    }
    .secondary {
      padding: var(--lp-space-2) var(--lp-space-4);
      font: inherit;
      color: var(--lp-color-text);
      background: transparent;
      border: 1px solid var(--lp-color-border);
      border-radius: var(--lp-radius-medium);
      cursor: pointer;
    }
  `,
})
export class ShortcutsHelpDialog {
  protected readonly state = inject(ShortcutsHelpState);
  protected readonly shortcuts = inject(Shortcuts);
  protected readonly describe = describeShortcut;
}
