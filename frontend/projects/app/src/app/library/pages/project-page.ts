import {
  ChangeDetectionStrategy,
  Component,
  effect,
  inject,
  input,
  signal,
  untracked,
} from '@angular/core';
import { RouterLink } from '@angular/router';
import { IonContent } from '@ionic/angular';
import { TranslocoPipe } from '@jsverse/transloco';
import { ProjectDirectory } from '../actions/project-directory';
import { toLibraryFailure, type LibraryFailure, type Project } from '../library-types';
import { AnimationList } from '../lists/animation-list';
import { Icon } from '../../ui/icon/icon';
import { StatusBanner } from '../../ui/status-banner/status-banner';

/** What the page knows of its project: still reading, found, or a failure to show. */
type ProjectLookup =
  | { readonly kind: 'loading' }
  | { readonly kind: 'found'; readonly project: Project }
  | { readonly kind: 'failed'; readonly failure: LibraryFailure };

const NOT_FOUND: LibraryFailure = { code: 'library.project_not_found', params: {} };

/** A project of the library, from `library/projects/:projectId`: its name and its animations. */
@Component({
  selector: 'lp-project-page',
  imports: [AnimationList, Icon, IonContent, RouterLink, StatusBanner, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <ion-content>
      <div class="lp-page">
        <a class="back" routerLink="/library">
          <lp-icon name="chevron-left" size="small" />
          {{ 'library.project.back' | transloco }}
        </a>
        @switch (lookup().kind) {
          @case ('found') {
            <h1 class="lp-page-header__title">{{ name() }}</h1>
            <lp-animation-list [projectId]="projectId()" />
          }
          @case ('failed') {
            <h1 class="lp-page-header__title">{{ 'library.title' | transloco }}</h1>
            <lp-status-banner variant="danger">
              {{ 'errors.' + failure().code | transloco: failure().params }}
            </lp-status-banner>
          }
          @default {
            <h1 class="lp-page-header__title">{{ 'library.title' | transloco }}</h1>
            <p role="status">{{ 'common.loading' | transloco }}</p>
          }
        }
      </div>
    </ion-content>
  `,
  styleUrl: './library-page.scss',
})
export class ProjectPage {
  private readonly directory = inject(ProjectDirectory);

  /** From the route `library/projects/:projectId`. */
  readonly projectId = input.required<string>();

  protected readonly lookup = signal<ProjectLookup>({ kind: 'loading' });

  constructor() {
    effect(() => {
      const id = this.projectId();
      untracked(() => void this.find(id));
    });
  }

  protected name(): string {
    const lookup = this.lookup();
    return lookup.kind === 'found' ? lookup.project.name : '';
  }

  protected failure(): LibraryFailure {
    const lookup = this.lookup();
    return lookup.kind === 'failed' ? lookup.failure : NOT_FOUND;
  }

  private async find(id: string): Promise<void> {
    this.lookup.set({ kind: 'loading' });
    try {
      const project = await this.directory.find(id);
      this.lookup.set(
        project ? { kind: 'found', project } : { kind: 'failed', failure: NOT_FOUND },
      );
    } catch (error: unknown) {
      this.lookup.set({ kind: 'failed', failure: toLibraryFailure(error) });
    }
  }
}
