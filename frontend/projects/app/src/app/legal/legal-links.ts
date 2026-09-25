import { ChangeDetectionStrategy, Component } from '@angular/core';
import { RouterLink } from '@angular/router';
import { TranslocoPipe } from '@jsverse/transloco';

/** A link of the footer: its page and the i18n key of its text. */
interface LegalLink {
  readonly page: string;
  readonly label: string;
}

const LEGAL_LINKS: readonly LegalLink[] = [
  { page: 'terms', label: 'legal.footer.terms' },
  { page: 'privacy', label: 'legal.footer.privacy' },
  { page: 'notice', label: 'legal.footer.notice' },
];

/**
 * The footer's content on the hosted app: links to the terms, the privacy policy and the legal
 * notice (H16), provided as `FOOTER_SLOT`.
 */
@Component({
  selector: 'lp-legal-links',
  imports: [RouterLink, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <ul class="links">
      @for (link of links; track link.page) {
        <li>
          <a [routerLink]="['/legal', link.page]">{{ link.label | transloco }}</a>
        </li>
      }
    </ul>
  `,
  styles: `
    .links {
      display: flex;
      flex-wrap: wrap;
      gap: var(--lp-space-4);
      padding: 0;
      margin: 0;
      list-style: none;
    }
    a {
      color: inherit;
      text-decoration: none;

      &:hover {
        text-decoration: underline;
      }
    }
  `,
})
export class LegalLinks {
  protected readonly links = LEGAL_LINKS;
}
