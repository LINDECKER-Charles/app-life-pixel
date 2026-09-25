import { ChangeDetectionStrategy, Component } from '@angular/core';
import { RouterLink } from '@angular/router';
import { IonContent } from '@ionic/angular';
import { TranslocoPipe } from '@jsverse/transloco';

/** Any path no route matches: says so, and leads back to the editor. */
@Component({
  selector: 'lp-not-found-page',
  imports: [IonContent, RouterLink, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <ion-content>
      <div class="page">
        <h1>{{ 'not_found.title' | transloco }}</h1>
        <p>{{ 'not_found.message' | transloco }}</p>
        <a routerLink="/editor">{{ 'not_found.back' | transloco }}</a>
      </div>
    </ion-content>
  `,
  styles: `
    .page {
      max-width: 40rem;
      padding: var(--lp-space-6) var(--lp-space-4);
      margin-inline: auto;
    }
    h1 {
      margin: 0 0 var(--lp-space-4);
      font-size: var(--lp-font-size-title);
    }
  `,
})
export class NotFoundPage {}
