import {
  ChangeDetectionStrategy,
  Component,
  effect,
  inject,
  input,
  signal,
  untracked,
} from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { RouterLink } from '@angular/router';
import { TranslocoPipe } from '@jsverse/transloco';
import { AnimationActions } from '../actions/animation-actions';
import { concernsProject, LibraryChanges, type LibraryChange } from '../changes/library-changes';
import { LIBRARY_STORE } from '../library-store';
import type { AnimationSummary } from '../library-types';
import { PagedList } from './paged-list';

/**
 * Animations of the library (accounts.md, H8) — across projects with a search field, or those of
 * one project —, 50 at a time with "Load more". Each opens in the editor, and can be renamed,
 * moved, duplicated or deleted. The list reads again when the library changes outside the app
 * (desktop.md, T3).
 */
@Component({
  selector: 'lp-animation-list',
  imports: [RouterLink, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './animation-list.html',
  styleUrl: './library-list.scss',
})
export class AnimationList {
  private readonly store = inject(LIBRARY_STORE);
  protected readonly actions = inject(AnimationActions);

  /** Only the animations of this project; all of them when absent, with a search field. */
  readonly projectId = input<string>();

  protected readonly typedQuery = signal('');
  private readonly appliedQuery = signal('');
  protected readonly list = new PagedList<AnimationSummary>((page) =>
    this.store.listAnimations({ projectId: this.projectId(), query: this.appliedQuery() }, page),
  );

  constructor() {
    effect(() => {
      this.projectId();
      untracked(() => void this.list.reload());
    });
    inject(LibraryChanges)
      .changes.pipe(takeUntilDestroyed())
      .subscribe((change) => this.onLibraryChanged(change));
  }

  private onLibraryChanged(change: LibraryChange): void {
    const projectId = this.projectId();
    if (projectId === undefined || concernsProject(change, projectId)) void this.list.reload();
  }

  protected onQueryInput(event: Event): void {
    if (event.target instanceof HTMLInputElement) this.typedQuery.set(event.target.value);
  }

  protected search(event: SubmitEvent): void {
    event.preventDefault();
    this.appliedQuery.set(this.typedQuery().trim());
    void this.list.reload();
  }
}
