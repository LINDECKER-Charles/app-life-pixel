import { ChangeDetectionStrategy, Component, inject, input, signal } from '@angular/core';
import { TranslocoPipe } from '@jsverse/transloco';
import { Icon } from '../ui/icon/icon';
import { Clipboard } from './clipboard';

/** Where copying stands: not yet, done, or refused by the system. */
type CopyStatus = 'idle' | 'copied' | 'failed';

/** A text to paste elsewhere — a command, a configuration —, shown whole with a copy button. */
@Component({
  selector: 'lp-copy-block',
  imports: [Icon, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <pre><code>{{ text() }}</code></pre>
    <div class="actions">
      <button type="button" class="lp-button lp-button--secondary" (click)="copy()">
        <lp-icon name="copy" />
        {{ copyLabel() | transloco }}
      </button>
      <span class="status" role="status">
        @switch (status()) {
          @case ('copied') {
            <span class="copied"><lp-icon name="check" size="small" /></span>
            {{ 'mcp.agents.copied' | transloco }}
          }
          @case ('failed') {
            <span class="failed"><lp-icon name="error" size="small" /></span>
            {{ 'mcp.agents.copy_failed' | transloco }}
          }
        }
      </span>
    </div>
  `,
  styles: `
    :host {
      display: flex;
      flex-direction: column;
      gap: var(--lp-space-2);
    }
    pre {
      padding: var(--lp-space-3);
      margin: 0;
      overflow-x: auto;
      font-family: var(--lp-font-family-mono);
      font-size: var(--lp-font-size-small);
      white-space: pre-wrap;
      overflow-wrap: anywhere;
      background: var(--lp-color-surface-soft);
      border: var(--lp-border-width) solid var(--lp-color-border-subtle);
      border-radius: var(--lp-radius-medium);
    }
    .actions {
      display: flex;
      flex-wrap: wrap;
      align-items: center;
      gap: var(--lp-space-3);
    }
    .status {
      display: inline-flex;
      gap: var(--lp-space-1);
      align-items: center;
      font-size: var(--lp-font-size-small);
    }
    .copied {
      color: var(--lp-color-success);
    }
    .failed {
      color: var(--lp-color-danger);
    }
  `,
})
export class CopyBlock {
  private readonly clipboard = inject(Clipboard);

  readonly text = input.required<string>();
  /** The i18n key of the button's label, which names what it copies. */
  readonly copyLabel = input.required<string>();

  protected readonly status = signal<CopyStatus>('idle');

  protected async copy(): Promise<void> {
    try {
      await this.clipboard.copy(this.text());
      this.status.set('copied');
    } catch {
      this.status.set('failed');
    }
  }
}
