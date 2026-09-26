import { Injectable } from '@angular/core';
import { type Observable, Subject } from 'rxjs';

/**
 * What changed in the library outside the app (desktop.md, T3): the projects and animations
 * whose files changed. Both empty means anything may have changed.
 */
export interface LibraryChange {
  readonly projectIds: readonly string[];
  readonly animationIds: readonly string[];
}

function concernsAnything(change: LibraryChange): boolean {
  return change.projectIds.length === 0 && change.animationIds.length === 0;
}

/** Whether `change` may concern the animation `id`. */
export function concernsAnimation(change: LibraryChange, id: string): boolean {
  return concernsAnything(change) || change.animationIds.includes(id);
}

/** Whether `change` may concern the project `id` or its animations. */
export function concernsProject(change: LibraryChange, id: string): boolean {
  return concernsAnything(change) || change.projectIds.includes(id);
}

/**
 * The library's changes made outside the app — by an agent through the CLI, or by hand —, as the
 * desktop's watcher reports them (desktop.md, T3); the hosted library reports none. Lists and the
 * open animation follow `changes`.
 */
@Injectable({ providedIn: 'root' })
export class LibraryChanges {
  private readonly subject = new Subject<LibraryChange>();

  readonly changes: Observable<LibraryChange> = this.subject.asObservable();

  /** Tells every follower that the library changed. */
  announce(change: LibraryChange): void {
    this.subject.next(change);
  }
}
