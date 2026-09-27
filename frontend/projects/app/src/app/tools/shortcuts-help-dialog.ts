import { ChangeDetectionStrategy, Component, inject } from '@angular/core';
import { IonModal } from '@ionic/angular';
import { TranslocoPipe } from '@jsverse/transloco';
import { idScope } from 'shared';
import { Shortcuts } from '../editor/shortcuts';
import { describeShortcut } from './shortcut-label';
import { ShortcutsHelpState } from './shortcuts-help-state';

/**
 * Lists every registered shortcut (editor.md, U4), each key combination a key cap, in the shared
 * dialog anatomy (`lp-dialog`); opened by the tool rail's `?` button and the `?` key.
 */
@Component({
  selector: 'lp-shortcuts-help-dialog',
  imports: [IonModal, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './shortcuts-help-dialog.html',
  styles: `
    /* The dialog clips the body's edges, which would crop the shared outset focus ring. */
    .lp-dialog__body:focus-visible {
      outline-offset: calc(-1 * var(--lp-focus-width));
    }
    .list {
      display: flex;
      flex-direction: column;
      gap: var(--lp-space-2);
      padding: 0;
      margin: 0;
      list-style: none;
    }
    .row {
      display: flex;
      justify-content: space-between;
      gap: var(--lp-space-4);
      align-items: baseline;
    }
    .keys {
      flex-shrink: 0;
      padding: 0 var(--lp-space-2);
      font-family: var(--lp-font-family-mono);
      font-size: var(--lp-font-size-small);
      color: var(--lp-color-text-muted);
      background: var(--lp-color-surface-soft);
      border: var(--lp-border-width) solid var(--lp-color-border-subtle);
      border-radius: var(--lp-radius-small);
    }
  `,
})
export class ShortcutsHelpDialog {
  protected readonly state = inject(ShortcutsHelpState);
  protected readonly shortcuts = inject(Shortcuts);
  protected readonly id = idScope('shortcuts-help-dialog');
  protected readonly describe = describeShortcut;
}
