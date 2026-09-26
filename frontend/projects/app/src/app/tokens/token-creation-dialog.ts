import { ChangeDetectionStrategy, Component, computed, model, output, signal } from '@angular/core';
import { IonModal } from '@ionic/angular';
import { TranslocoPipe } from '@jsverse/transloco';
import type { CreatedAccessToken } from 'shared';
import { ModalLabel } from '../library/save/modal-label';
import { CreatedToken } from './created-token';
import { TokenCreationForm } from './token-creation-form';

/**
 * Creates a token, then shows its secret once (mcp-cli.md, A3): closing the dialog forgets the
 * secret for good.
 */
@Component({
  selector: 'lp-token-creation-dialog',
  imports: [CreatedToken, IonModal, ModalLabel, TokenCreationForm, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './token-creation-dialog.html',
  styleUrl: './token-dialog.scss',
})
export class TokenCreationDialog {
  /** Whether the dialog shows. */
  readonly open = model(false);
  /** A token was created: the list shows it. */
  readonly created = output<CreatedAccessToken>();

  protected readonly token = signal<CreatedAccessToken | undefined>(undefined);
  protected readonly titleKey = computed(() =>
    this.token() ? 'tokens.created.title' : 'tokens.create.title',
  );

  protected onCreated(token: CreatedAccessToken): void {
    this.token.set(token);
    this.created.emit(token);
  }

  /** Closes the dialog, forgetting the secret. */
  protected close(): void {
    this.token.set(undefined);
    this.open.set(false);
  }
}
