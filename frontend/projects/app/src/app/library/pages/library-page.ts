import { ChangeDetectionStrategy, Component } from '@angular/core';
import { TranslocoPipe } from '@jsverse/transloco';
import { AnimationList } from '../lists/animation-list';
import { ProjectList } from '../lists/project-list';

/** The library (accounts.md, H8): the projects with their counts, and every animation. */
@Component({
  selector: 'lp-library-page',
  imports: [AnimationList, ProjectList, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <main class="page">
      <h1>{{ 'library.title' | transloco }}</h1>
      <section aria-labelledby="library-projects-heading">
        <h2 id="library-projects-heading">{{ 'library.projects.heading' | transloco }}</h2>
        <lp-project-list />
      </section>
      <section aria-labelledby="library-animations-heading">
        <h2 id="library-animations-heading">{{ 'library.animations.heading' | transloco }}</h2>
        <lp-animation-list />
      </section>
    </main>
  `,
  styleUrl: './library-page.scss',
})
export class LibraryPage {}
