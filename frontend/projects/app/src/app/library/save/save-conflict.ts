import { inject, Injectable } from '@angular/core';
import { TranslocoService } from '@jsverse/transloco';
import { EngineStore } from '../../engine/engine-store';
import { LIBRARY_STORE } from '../library-store';
import type { AnimationSummary } from '../library-types';
import { AnimationLoader } from '../open/animation-loader';
import type { ConflictChoice } from './save-prompts';

/** The saved animation a save collided on, and the document the editor tried to save. */
export interface Conflict {
  readonly id: string;
  readonly projectId: string;
  readonly document: Uint8Array;
}

/**
 * The three ways out of `document.version_conflict` (accounts.md, H8): reload the saved version,
 * overwrite it — read its current version, then save over it —, or save the work as a copy.
 * Each resolves to the animation now saved to mark the editor's work saved, or to `null` when
 * the editor already holds the saved version; failures reject as the store's do.
 */
@Injectable({ providedIn: 'root' })
export class SaveConflict {
  private readonly store = inject(LIBRARY_STORE);
  private readonly engine = inject(EngineStore);
  private readonly loader = inject(AnimationLoader);
  private readonly transloco = inject(TranslocoService);

  resolve(conflict: Conflict, choice: ConflictChoice): Promise<AnimationSummary | null> {
    switch (choice) {
      case 'reload':
        return this.reload(conflict);
      case 'overwrite':
        return this.overwrite(conflict);
      case 'copy':
        return this.saveCopy(conflict);
    }
  }

  private async reload(conflict: Conflict): Promise<null> {
    await this.loader.load(conflict.id);
    return null;
  }

  private async overwrite(conflict: Conflict): Promise<AnimationSummary> {
    const { summary } = await this.store.openDocument(conflict.id);
    return this.store.saveDocument(conflict.id, conflict.document, summary.version);
  }

  /**
   * A new animation beside the original, titled `library.copy_of`; the editor follows the copy.
   * A copy the store could not retitle keeps the original's title.
   */
  private async saveCopy(conflict: Conflict): Promise<AnimationSummary> {
    const created = await this.store.createAnimation(conflict.projectId, conflict.document);
    const title = this.transloco.translate('library.copy_of', { name: created.title });
    try {
      const renamed = await this.store.renameAnimation(created.id, title, created.version);
      await this.engine.apply({ kind: 'setTitle', title: renamed.title });
      return renamed;
    } catch {
      return created;
    }
  }
}
