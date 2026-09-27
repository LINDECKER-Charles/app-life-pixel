import { inject, Injectable } from '@angular/core';
import { TranslocoService } from '@jsverse/transloco';
import { EngineStore } from '../../engine/engine-store';
import { CurrentAnimation } from '../current-animation';
import { LIBRARY_STORE } from '../library-store';
import type { AnimationSummary } from '../library-types';
import type { PagedList } from '../lists/paged-list';
import { LibraryPrompts } from './library-prompts';
import { ProjectDirectory } from './project-directory';

type AnimationList = PagedList<AnimationSummary>;

/**
 * What the library pages do to animations (accounts.md, H8): rename, move, duplicate — titled
 * `library.copy_of` — and delete, after a confirmation. Each keeps `list` in step and records a
 * failure on it; when the animation is the editor's, `CurrentAnimation` follows too.
 */
@Injectable({ providedIn: 'root' })
export class AnimationActions {
  private readonly store = inject(LIBRARY_STORE);
  private readonly prompts = inject(LibraryPrompts);
  private readonly directory = inject(ProjectDirectory);
  private readonly current = inject(CurrentAnimation);
  private readonly engine = inject(EngineStore);
  private readonly transloco = inject(TranslocoService);

  /** Retitles the animation: a save of its document, under the version the list read. */
  async rename(animation: AnimationSummary, list: AnimationList): Promise<void> {
    const title = await this.prompts.askName({
      titleKey: 'library.animation.rename_title',
      labelKey: 'library.animation.title_label',
      value: animation.title,
    });
    if (title === null || title === animation.title) return;
    await this.run(list, async () => {
      const renamed = await this.store.renameAnimation(animation.id, title, animation.version);
      list.replace(renamed);
      await this.followRename(renamed);
    });
  }

  /** Moves the animation to the project picked; a list of one project lets it go. */
  async move(animation: AnimationSummary, list: AnimationList, byProject = false): Promise<void> {
    await this.run(list, async () => {
      const projects = await this.directory.all();
      const projectId = await this.prompts.pickProject(projects, animation.projectId);
      if (projectId === null || projectId === animation.projectId) return;
      const moved = await this.store.moveAnimation(animation.id, projectId);
      if (byProject) list.remove(moved.id);
      else list.replace(moved);
      if (this.current.holds(moved.id)) this.current.setSaved(moved);
    });
  }

  async duplicate(animation: AnimationSummary, list: AnimationList): Promise<void> {
    const title = this.transloco.translate('library.copy_of', { name: animation.title });
    await this.run(list, async () =>
      list.prepend(await this.store.duplicateAnimation(animation.id, title)),
    );
  }

  /** Deletes the animation; the editor keeps it as unsaved work if it holds it. */
  async delete(animation: AnimationSummary, list: AnimationList): Promise<void> {
    const confirmed = await this.prompts.confirmDestroy({
      titleKey: 'library.animation.delete_title',
      messageKey: 'library.animation.delete_message',
      name: animation.title,
    });
    if (!confirmed) return;
    await this.run(list, async () => {
      await this.store.deleteAnimation(animation.id);
      list.remove(animation.id);
      if (this.current.holds(animation.id)) this.current.setUnsaved();
    });
  }

  /**
   * The editor's animation, renamed in the library: its new version and title follow, and work
   * that was saved stays saved.
   */
  private async followRename(renamed: AnimationSummary): Promise<void> {
    if (!this.current.holds(renamed.id)) return;
    const wasSaved = !this.engine.hasUnsavedWork();
    this.current.setSaved(renamed);
    await this.engine.apply({ kind: 'setTitle', title: renamed.title });
    if (wasSaved) await this.engine.markSaved();
  }

  private async run(list: AnimationList, action: () => Promise<void>): Promise<void> {
    try {
      await action();
    } catch (error: unknown) {
      list.fail(error);
    }
  }
}
