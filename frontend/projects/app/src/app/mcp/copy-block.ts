import { ChangeDetectionStrategy, Component, inject, input, signal } from '@angular/core';
import { TranslocoPipe } from '@jsverse/transloco';
import { Clipboard } from './clipboard';

/** Where copying stands: not yet, done, or refused by the system. */
type CopyStatus = 'idle' | 'copied' | 'failed';

/** A text to paste elsewhere — a command, a configuration —, shown whole with a copy button. */
@Component({
  selector: 'lp-copy-block',
  imports: [TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <pre><code>{{ text() }}</code></pre>
    <div class="actions">
      <button type="button" (click)="copy()">{{ copyLabel() | transloco }}</button>
      <span role="status">
        @switch (status()) {
          @case ('copied') {
            {{ 'mcp.agents.copied' | transloco }}
          }
          @case ('failed') {
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
      font-size: var(--lp-font-size-small);
      white-space: pre-wrap;
      overflow-wrap: anywhere;
      background: var(--lp-color-surface);
      border: 1px solid var(--lp-color-border);
      border-radius: var(--lp-radius-small);
    }
    .actions {
      display: flex;
      align-items: center;
      gap: var(--lp-space-3);
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
