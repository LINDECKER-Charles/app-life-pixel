import {
  ChangeDetectionStrategy,
  Component,
  computed,
  effect,
  inject,
  input,
  signal,
  untracked,
  ViewEncapsulation,
} from '@angular/core';
import { toSignal } from '@angular/core/rxjs-interop';
import { IonContent } from '@ionic/angular';
import { TranslocoService } from '@jsverse/transloco';
import { parse as parseMarkdown } from 'marked';
import { LegalTextLoader } from './legal-text-loader';

/**
 * `legal/:page` — `page` one of `terms`, `privacy` or `notice` (H16): renders that page's
 * markdown in the active language, English when the active language has none, as sanitized HTML.
 * The text brings its own `h1`. Unencapsulated styles, under `.lp-legal`, reach that HTML.
 */
@Component({
  selector: 'lp-legal-page',
  imports: [IonContent],
  changeDetection: ChangeDetectionStrategy.OnPush,
  encapsulation: ViewEncapsulation.None,
  template: `
    <ion-content>
      <div class="lp-page lp-page--reading">
        <article class="lp-legal" [innerHTML]="html()"></article>
      </div>
    </ion-content>
  `,
  styles: `
    .lp-legal {
      max-width: var(--lp-measure-copy);
      line-height: var(--lp-font-line-height);

      h1 {
        margin: 0 0 var(--lp-space-5);
        font-size: var(--lp-font-size-heading);
        line-height: var(--lp-font-line-height-heading);
      }
      h2 {
        margin: var(--lp-space-5) 0 var(--lp-space-3);
        font-size: var(--lp-font-size-title);
        line-height: var(--lp-font-line-height-heading);
      }
      h3 {
        margin: var(--lp-space-4) 0 var(--lp-space-2);
        font-size: var(--lp-font-size-large);
      }
      p,
      ul,
      ol {
        margin: 0 0 var(--lp-space-3);
      }
      a {
        overflow-wrap: anywhere;
      }
    }
  `,
})
export class LegalPage {
  private readonly loader = inject(LegalTextLoader);
  private readonly transloco = inject(TranslocoService);
  private readonly language = toSignal(this.transloco.langChanges$, {
    initialValue: this.transloco.getActiveLang(),
  });
  private readonly text = signal('');
  private requestId = 0;

  readonly page = input.required<string>();

  protected readonly html = computed(() => parseMarkdown(this.text(), { async: false }));

  constructor() {
    effect(() => {
      const page = this.page();
      const language = this.language();
      untracked(() => void this.refresh(page, language));
    });
  }

  private async refresh(page: string, language: string): Promise<void> {
    const id = ++this.requestId;
    const text = await this.loader.load(language, page);
    if (id === this.requestId) this.text.set(text);
  }
}
