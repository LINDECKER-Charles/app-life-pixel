import { NgComponentOutlet } from '@angular/common';
import { ChangeDetectionStrategy, Component, inject } from '@angular/core';
import { FOOTER_SLOT } from './footer-slot';

/**
 * The page's footer, quiet under the routed page: what `FOOTER_SLOT` provides, in muted small
 * text; nothing when it provides nothing.
 */
@Component({
  selector: 'lp-app-footer',
  imports: [NgComponentOutlet],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    @if (content) {
      <footer class="footer">
        <ng-container *ngComponentOutlet="content" />
      </footer>
    }
  `,
  styles: `
    .footer {
      padding: var(--lp-space-2) var(--lp-space-4);
      font-size: var(--lp-font-size-small);
      color: var(--lp-color-text-muted);
      background: var(--lp-color-background);
      border-top: var(--lp-border-width) solid var(--lp-color-border-subtle);
    }
  `,
})
export class AppFooter {
  protected readonly content = inject(FOOTER_SLOT, { optional: true });
}
