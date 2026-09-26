import { ChangeDetectionStrategy, Component } from '@angular/core';
import { IonContent } from '@ionic/angular';
import { TranslocoPipe } from '@jsverse/transloco';
import { CreateAnimationButton } from '../actions/create-animation-button';
import { AnimationList } from '../lists/animation-list';
import { ProjectList } from '../lists/project-list';

/** The library (accounts.md, H8): the projects with their counts, and every animation. */
@Component({
  selector: 'lp-library-page',
  imports: [AnimationList, CreateAnimationButton, IonContent, ProjectList, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <ion-content>
      <div class="lp-page">
        <header class="lp-page-header">
          <div class="lp-page-header__text">
            <h1 class="lp-page-header__title">{{ 'library.title' | transloco }}</h1>
            <p class="lp-page-header__lead">{{ 'library.lead' | transloco }}</p>
          </div>
          <div class="lp-page-header__actions"><lp-create-animation-button /></div>
        </header>
        <section class="lp-section" aria-labelledby="library-projects-heading">
          <h2 id="library-projects-heading">{{ 'library.projects.heading' | transloco }}</h2>
          <lp-project-list />
        </section>
        <section class="lp-section" aria-labelledby="library-animations-heading">
          <h2 id="library-animations-heading">{{ 'library.animations.heading' | transloco }}</h2>
          <lp-animation-list />
        </section>
      </div>
    </ion-content>
  `,
})
export class LibraryPage {}
