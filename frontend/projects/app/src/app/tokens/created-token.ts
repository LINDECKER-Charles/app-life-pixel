import {
  ChangeDetectionStrategy,
  Component,
  computed,
  DestroyRef,
  inject,
  input,
  output,
  signal,
} from '@angular/core';
import { TranslocoPipe } from '@jsverse/transloco';
import type { CreatedAccessToken } from 'shared';
import { mcpCommand } from './token-values';

const COPIED_DURATION_MS = 2000;

/** What a copy button copies. */
type Copied = 'token' | 'command';

/**
 * A token just created (mcp-cli.md, A3): its secret, shown this once, and the `claude mcp add`
 * command that registers the endpoint with it, each with a copy button.
 */
@Component({
  selector: 'lp-created-token',
  imports: [TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './created-token.html',
  styleUrl: './token-dialog.scss',
})
export class CreatedToken {
  private copiedTimeout: ReturnType<typeof setTimeout> | undefined;

  /** The token, its secret in it. */
  readonly token = input.required<CreatedAccessToken>();
  /** The person has kept the secret. */
  readonly done = output();

  protected readonly command = computed(() => mcpCommand(this.token()));
  protected readonly copied = signal<Copied | undefined>(undefined);

  constructor() {
    inject(DestroyRef).onDestroy(() => clearTimeout(this.copiedTimeout));
  }

  protected async copy(what: Copied): Promise<void> {
    const text = what === 'token' ? this.token().token : this.command();
    await navigator.clipboard.writeText(text);
    this.copied.set(what);
    clearTimeout(this.copiedTimeout);
    this.copiedTimeout = setTimeout(() => this.copied.set(undefined), COPIED_DURATION_MS);
  }
}
