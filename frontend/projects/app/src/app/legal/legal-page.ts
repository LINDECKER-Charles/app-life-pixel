import {
  ChangeDetectionStrategy,
  Component,
  computed,
  effect,
  inject,
  input,
  signal,
  untracked,
} from '@angular/core';
import { toSignal } from '@angular/core/rxjs-interop';
import { IonContent } from '@ionic/angular';
import { TranslocoService } from '@jsverse/transloco';
import { parse as parseMarkdown } from 'marked';
import { LegalTextLoader } from './legal-text-loader';

/**
 * `legal/:page` — `page` one of `terms`, `privacy` or `notice` (H16): renders that page's
 * markdown in the active language, English when the active language has none, as sanitized HTML.
 */
@Component({
  selector: 'lp-legal-page',
  imports: [IonContent],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <ion-content>
      <div class="page" [innerHTML]="html()"></div>
    </ion-content>
  `,
  styles: `
    .page {
      max-width: 40rem;
      padding: var(--lp-space-6) var(--lp-space-4);
      margin-inline: auto;
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
