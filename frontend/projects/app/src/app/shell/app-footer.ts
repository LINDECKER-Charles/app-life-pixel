import { NgComponentOutlet } from '@angular/common';
import { ChangeDetectionStrategy, Component, inject } from '@angular/core';
import { FOOTER_SLOT } from './footer-slot';

/** The page's footer, holding what `FOOTER_SLOT` provides; nothing when it provides nothing. */
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
      background: var(--lp-color-surface);
      border-top: 1px solid var(--lp-color-border);
    }
  `,
})
export class AppFooter {
  protected readonly content = inject(FOOTER_SLOT, { optional: true });
}
