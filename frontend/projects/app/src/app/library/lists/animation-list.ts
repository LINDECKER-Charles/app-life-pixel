import {
  ChangeDetectionStrategy,
  Component,
  effect,
  ElementRef,
  inject,
  input,
  signal,
  untracked,
  viewChild,
} from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { RouterLink } from '@angular/router';
import { TranslocoPipe } from '@jsverse/transloco';
import { EmptyState } from '../../ui/empty-state/empty-state';
import { Icon } from '../../ui/icon/icon';
import { MenuButton } from '../../ui/menu/menu-button';
import { AnimationActions } from '../actions/animation-actions';
import { CreateAnimationButton } from '../actions/create-animation-button';
import { concernsProject, LibraryChanges, type LibraryChange } from '../changes/library-changes';
import { LIBRARY_STORE } from '../library-store';
import type { AnimationSummary } from '../library-types';
import { ListStatus } from './list-status';
import { PagedList } from './paged-list';

/**
 * Animations of the library (accounts.md, H8) — across projects with a search field, or those of
 * one project —, 50 at a time with "Load more". Each opens in the editor; its menu renames,
 * moves or duplicates it, and a separate button deletes it. An empty library or project, a search
 * without results and a failure each say so differently. The list reads again when the library
 * changes outside the app (desktop.md, T3).
 */
@Component({
  selector: 'lp-animation-list',
  imports: [
    CreateAnimationButton,
    EmptyState,
    Icon,
    ListStatus,
    MenuButton,
    RouterLink,
    TranslocoPipe,
  ],
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
  /** The query the list shows the results of: the one last submitted. */
  protected readonly appliedQuery = signal('');
  private readonly searchField = viewChild<ElementRef<HTMLInputElement>>('searchField');
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

  /** Shows every animation again, and puts the focus back in the search field. */
  protected clearSearch(): void {
    this.typedQuery.set('');
    this.appliedQuery.set('');
    void this.list.reload();
    this.searchField()?.nativeElement.focus();
  }
}
