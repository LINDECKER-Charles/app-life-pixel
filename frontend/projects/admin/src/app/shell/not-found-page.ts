import { ChangeDetectionStrategy, Component, inject } from '@angular/core';
import { RouterLink } from '@angular/router';
import { TranslocoPipe } from '@jsverse/transloco';
import { ViewStateStore } from '../core/view-state-store';

/** An address the console does not know: back to the overview. */
@Component({
  selector: 'lp-not-found-page',
  imports: [RouterLink, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <h1>{{ 'admin.not_found.title' | transloco }}</h1>
    <p>
      <a routerLink="/" [queryParams]="params()">{{ 'admin.not_found.back' | transloco }}</a>
    </p>
  `,
})
export class NotFoundPage {
  protected readonly params = inject(ViewStateStore).params;
}
