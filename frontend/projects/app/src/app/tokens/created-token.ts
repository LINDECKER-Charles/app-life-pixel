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
import { claudeHttpCommand, codexHttpCommand, tokenVariable } from './token-values';

const COPIED_DURATION_MS = 2000;

/** What a copy button copies. */
type Copied = 'token' | 'claude' | 'codex';

/** A text the dialog offers to copy, under its label: the secret, or a command that uses it. */
interface CopyField {
  readonly what: Copied;
  readonly label: string;
  readonly copyLabel: string;
  /** What to do with the text, when it needs saying: an i18n key and its parameters. */
  readonly hint?: { readonly key: string; readonly params?: Readonly<Record<string, string>> };
  readonly text: string;
}

/** The outcome of the last copy, beside its button. */
interface CopyState {
  readonly what: Copied;
  readonly failed: boolean;
}

/**
 * A token just created (mcp-cli.md, A3): its secret, shown this once, and the `claude mcp add` and
 * `codex mcp add` commands that register the endpoint with it, each with a copy button. A copy the
 * browser refuses selects the text instead, to copy by hand (design-system/docs/patterns.md,
 * "Account, access and privacy").
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

  protected readonly fields = computed(() => copyFields(this.token()));
  protected readonly copied = signal<CopyState | undefined>(undefined);

  constructor() {
    inject(DestroyRef).onDestroy(() => clearTimeout(this.copiedTimeout));
  }

  protected async copy({ what, text }: CopyField, shown: HTMLElement): Promise<void> {
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

// Literal keys, so that the i18n check sees each one used.
/** The secret, then the command of each client that registers the endpoint with it. */
function copyFields(created: CreatedAccessToken): readonly CopyField[] {
  const variable = tokenVariable(created.mcp.serverName);
  return [
    {
      what: 'token',
      label: 'tokens.created.token_label',
      copyLabel: 'tokens.created.copy_token',
      text: created.token,
    },
    {
      what: 'claude',
      label: 'tokens.created.claude.label',
      copyLabel: 'tokens.created.claude.copy',
      hint: { key: 'tokens.created.claude.hint' },
      text: claudeHttpCommand(created),
    },
    {
      what: 'codex',
      label: 'tokens.created.codex.label',
      copyLabel: 'tokens.created.codex.copy',
      hint: { key: 'tokens.created.codex.hint', params: { variable } },
      text: codexHttpCommand(created),
    },
  ];
}

/** Selects the text of `element`, ready for the system's own copy. */
function selectText(element: HTMLElement): void {
  const range = document.createRange();
  range.selectNodeContents(element);
  const selection = window.getSelection();
  selection?.removeAllRanges();
  selection?.addRange(range);
}
