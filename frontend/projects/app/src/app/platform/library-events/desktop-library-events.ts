import { inject, Injectable } from '@angular/core';
import { listen } from '@tauri-apps/api/event';
import { type LibraryChange, LibraryChanges } from '../../library/changes/library-changes';
import { OpenAnimationRefresh } from '../../library/changes/open-animation-refresh';

/** The event the desktop's library watcher emits, with a `LibraryChange` (desktop.md, T3). */
export const LIBRARY_CHANGED_EVENT = 'library-changed';

/**
 * Brings the desktop watcher's `library-changed` events into the app (desktop.md, T3): what an
 * agent writes through the bundled CLI refreshes the lists, and the open animation follows.
 */
@Injectable({ providedIn: 'root' })
export class DesktopLibraryEvents {
  private readonly changes = inject(LibraryChanges);
  private readonly refresh = inject(OpenAnimationRefresh);

  /** Starts following the library for the rest of the app's life; resolves once listening. */
  async follow(): Promise<void> {
    this.refresh.follow();
    try {
      await listen<LibraryChange>(LIBRARY_CHANGED_EVENT, ({ payload }) =>
        this.changes.announce(payload),
      );
    } catch {
      // Without the events the app still works: its lists read the library on each visit.
    }
  }
}
