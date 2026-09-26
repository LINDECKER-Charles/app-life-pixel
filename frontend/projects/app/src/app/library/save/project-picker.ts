import {
  ChangeDetectionStrategy,
  Component,
  computed,
  inject,
  output,
  signal,
} from '@angular/core';
import { TranslocoPipe } from '@jsverse/transloco';
import { LIBRARY_STORE } from '../library-store';
import type { Project } from '../library-types';
import { PagedList } from '../lists/paged-list';
import type { ProjectChoice } from './save-prompts';

/** The picker's value for "a new project", which no project id can take. */
const NEW_PROJECT = '';

/**
 * Where unsaved work goes on its first save (accounts.md, H8): a project of the library, 50 at a
 * time with "Load more", or a new one named here — the choice when the library has none.
 */
@Component({
  selector: 'lp-project-picker',
  imports: [TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <form class="picker" novalidate (submit)="submit($event)">
      <fieldset>
        <legend>{{ 'library.save.project.legend' | transloco }}</legend>
        @for (project of projects.items(); track project.id) {
          <label class="option">
            <input
              type="radio"
              name="project"
              [value]="project.id"
              [checked]="selected() === project.id"
              (change)="selected.set(project.id)"
            />
            {{ project.name }}
          </label>
        }
        @if (projects.hasMore()) {
          <button type="button" [disabled]="projects.loading()" (click)="projects.loadMore()">
            {{ 'library.load_more' | transloco }}
          </button>
        }
        <label class="option">
          <input
            type="radio"
            name="project"
            [value]="newProject"
            [checked]="selected() === newProject"
            (change)="selected.set(newProject)"
          />
          {{ 'library.save.project.new' | transloco }}
        </label>
        @if (selected() === newProject) {
          <label class="field">
            {{ 'library.project.name_label' | transloco }}
            <input
              type="text"
              name="name"
              required
              [value]="name()"
              (input)="onNameInput($event)"
            />
          </label>
        }
      </fieldset>
      @if (projects.failure(); as failure) {
        <p role="alert">{{ 'errors.' + failure.code | transloco: failure.params }}</p>
      }
      <div class="actions">
        <button type="button" (click)="cancelled.emit()">
          {{ 'common.cancel' | transloco }}
        </button>
        <button type="submit" class="primary" [disabled]="!canSubmit()">
          {{ 'library.save.project.submit' | transloco }}
        </button>
      </div>
    </form>
  `,
  styleUrl: '../library-dialog.scss',
})
export class ProjectPicker {
  private readonly store = inject(LIBRARY_STORE);

  readonly chosen = output<ProjectChoice>();
  readonly cancelled = output<void>();

  protected readonly newProject = NEW_PROJECT;
  protected readonly projects = new PagedList<Project>((page) => this.store.listProjects(page));
  protected readonly selected = signal(NEW_PROJECT);
  protected readonly name = signal('');
  protected readonly canSubmit = computed(
    () => this.selected() !== NEW_PROJECT || this.name().trim() !== '',
  );

  constructor() {
    void this.projects.reload().then(() => {
      const first = this.projects.items()[0];
      if (first) this.selected.set(first.id);
    });
  }

  protected onNameInput(event: Event): void {
    if (event.target instanceof HTMLInputElement) this.name.set(event.target.value);
  }

  protected submit(event: SubmitEvent): void {
    event.preventDefault();
    if (!this.canSubmit()) return;
    const selected = this.selected();
    this.chosen.emit(
      selected === NEW_PROJECT ? { newProjectName: this.name().trim() } : { projectId: selected },
    );
  }
}
