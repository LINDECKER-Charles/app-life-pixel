import { NgTemplateOutlet } from '@angular/common';
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
import { Icon } from '../ui/icon/icon';
import { StatusBanner } from '../ui/status-banner/status-banner';
import { mcpCommand } from './token-values';

const COPIED_DURATION_MS = 2000;

/** What a copy button copies. */
type Copied = 'token' | 'command';

/** The outcome of the last copy, beside its button. */
interface CopyState {
  readonly what: Copied;
  readonly failed: boolean;
}

/**
 * A token just created (mcp-cli.md, A3): its secret, shown this once, and the `claude mcp add`
 * command that registers the endpoint with it, each with a copy button. A copy the browser
 * refuses selects the text instead, to copy by hand (design-system/docs/patterns.md, "Account,
 * access and privacy").
 */
@Component({
  selector: 'lp-created-token',
  imports: [Icon, NgTemplateOutlet, StatusBanner, TranslocoPipe],
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
  protected readonly copied = signal<CopyState | undefined>(undefined);

  constructor() {
    inject(DestroyRef).onDestroy(() => clearTimeout(this.copiedTimeout));
  }

  protected async copy(what: Copied, shown: HTMLElement): Promise<void> {
    const text = what === 'token' ? this.token().token : this.command();
    clearTimeout(this.copiedTimeout);
    try {
      await navigator.clipboard.writeText(text);
    } catch {
      selectText(shown);
      this.copied.set({ what, failed: true });
      return;
    }
    this.copied.set({ what, failed: false });
    this.copiedTimeout = setTimeout(() => this.copied.set(undefined), COPIED_DURATION_MS);
  }
}

/** Selects the text of `element`, ready for the system's own copy. */
function selectText(element: HTMLElement): void {
  const range = document.createRange();
  range.selectNodeContents(element);
  const selection = window.getSelection();
  selection?.removeAllRanges();
  selection?.addRange(range);
}
