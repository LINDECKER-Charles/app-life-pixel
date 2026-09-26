import { inject, Injectable } from '@angular/core';
import { TranslocoService } from '@jsverse/transloco';
import { CurrentAnimation } from '../current-animation';
import { LIBRARY_STORE } from '../library-store';
import type { Project } from '../library-types';
import type { PagedList } from '../lists/paged-list';
import { LibraryPrompts } from './library-prompts';

/**
 * What the library page does to projects (accounts.md, H8): create, rename, duplicate — named
 * `library.copy_of` — and delete, after a confirmation. Each keeps `list` in step, and records a
 * failure on it.
 */
@Injectable({ providedIn: 'root' })
export class ProjectActions {
  private readonly store = inject(LIBRARY_STORE);
  private readonly prompts = inject(LibraryPrompts);
  private readonly current = inject(CurrentAnimation);
  private readonly transloco = inject(TranslocoService);

  /** Creates a project named `name`; resolves to whether it was created. */
  async create(name: string, list: PagedList<Project>): Promise<boolean> {
    try {
      list.prepend(await this.store.createProject(name));
      return true;
    } catch (error: unknown) {
      list.fail(error);
      return false;
    }
  }

  async rename(project: Project, list: PagedList<Project>): Promise<void> {
    const name = await this.prompts.askName({
      titleKey: 'library.project.rename_title',
      labelKey: 'library.project.name_label',
      value: project.name,
    });
    if (name === null || name === project.name) return;
    await this.run(list, async () =>
      list.replace(await this.store.renameProject(project.id, name)),
    );
  }

  async duplicate(project: Project, list: PagedList<Project>): Promise<void> {
    const name = this.transloco.translate('library.copy_of', { name: project.name });
    await this.run(list, async () =>
      list.prepend(await this.store.duplicateProject(project.id, name)),
    );
  }

  /** Deletes the project and its animations; the editor keeps its work, now unsaved. */
  async delete(project: Project, list: PagedList<Project>): Promise<void> {
    const confirmed = await this.prompts.confirmDestroy({
      titleKey: 'library.project.delete_title',
      messageKey: 'library.project.delete_message',
      name: project.name,
    });
    if (!confirmed) return;
    await this.run(list, async () => {
      await this.store.deleteProject(project.id);
      list.remove(project.id);
      const state = this.current.state();
      if (state.kind === 'saved' && state.projectId === project.id) this.current.setUnsaved();
    });
  }

  private async run(list: PagedList<Project>, action: () => Promise<void>): Promise<void> {
    try {
      await action();
    } catch (error: unknown) {
      list.fail(error);
    }
  }
}
