import { ChangeDetectionStrategy, Component } from '@angular/core';
import { RouterLink } from '@angular/router';
import { IonContent } from '@ionic/angular';
import { TranslocoPipe } from '@jsverse/transloco';

/** Any path no route matches: says so, and leads back to the editor. No mascot: an error. */
@Component({
  selector: 'lp-not-found-page',
  imports: [IonContent, RouterLink, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <ion-content>
      <div class="lp-page lp-page--reading">
        <header class="lp-page-header">
          <div class="lp-page-header__text">
            <h1 class="lp-page-header__title">{{ 'not_found.title' | transloco }}</h1>
            <p class="lp-page-header__lead">{{ 'not_found.message' | transloco }}</p>
          </div>
        </header>
        <p>
          <a class="lp-button lp-button--primary" routerLink="/editor">
            {{ 'not_found.back' | transloco }}
          </a>
        </p>
      </div>
    </ion-content>
  `,
})
export class NotFoundPage {}
