import { ChangeDetectionStrategy, Component, inject, signal } from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { RouterLink } from '@angular/router';
import { TranslocoPipe } from '@jsverse/transloco';
import { ProjectActions } from '../actions/project-actions';
import { LibraryChanges } from '../changes/library-changes';
import { LIBRARY_STORE } from '../library-store';
import type { Project } from '../library-types';
import { PagedList } from './paged-list';

/**
 * The library's projects with their counts (accounts.md, H8), 50 at a time with "Load more": a
 * form creates one; each can be opened, renamed, duplicated or deleted. The list reads again when
 * the library changes outside the app (desktop.md, T3).
 */
@Component({
  selector: 'lp-project-list',
  imports: [RouterLink, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './project-list.html',
  styleUrl: './library-list.scss',
})
export class ProjectList {
  private readonly store = inject(LIBRARY_STORE);
  protected readonly actions = inject(ProjectActions);

  protected readonly list = new PagedList<Project>((page) => this.store.listProjects(page));
  protected readonly newName = signal('');

  constructor() {
    void this.list.reload();
    inject(LibraryChanges)
      .changes.pipe(takeUntilDestroyed())
      .subscribe(() => void this.list.reload());
  }

  protected onNameInput(event: Event): void {
    if (event.target instanceof HTMLInputElement) this.newName.set(event.target.value);
  }

  protected async create(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    const name = this.newName().trim();
    if (name === '') return;
    if (await this.actions.create(name, this.list)) this.newName.set('');
  }
}
