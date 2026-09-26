import { ChangeDetectionStrategy, Component, computed, inject } from '@angular/core';
import { RouterLink } from '@angular/router';
import { IonModal } from '@ionic/angular';
import { TranslocoPipe } from '@jsverse/transloco';
import { ModalLabel } from './modal-label';
import { ProjectPicker } from './project-picker';
import { SavePrompts } from './save-prompts';

/** The heading of each question saving asks. */
const TITLE_KEYS = {
  'sign-in': 'library.save.sign_in.title',
  project: 'library.save.project.title',
  conflict: 'library.save.conflict.title',
  quota: 'library.save.quota.title',
  failure: 'library.save.failure.title',
} as const;

/** The role of a dialog closed by its backdrop or Escape, rather than by a new question. */
const BACKDROP_ROLE = 'backdrop';

/**
 * The questions saving asks (accounts.md, H8), one at a time as `SavePrompts` holds them: sign in
 * or up, the project of a first save, the way out of a conflict, the quota reached, a failure.
 * Closing it any way dismisses the question.
 */
@Component({
  selector: 'lp-save-dialog',
  imports: [IonModal, ModalLabel, ProjectPicker, RouterLink, TranslocoPipe],
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './save-dialog.html',
  styleUrl: '../library-dialog.scss',
})
export class SaveDialog {
  protected readonly prompts = inject(SavePrompts);

  protected readonly kind = computed(() => this.prompts.current()?.kind ?? null);
  protected readonly titleKey = computed(() => {
    const kind = this.kind();
    return kind === null ? TITLE_KEYS.failure : TITLE_KEYS[kind];
  });
  protected readonly quota = computed(() => {
    const prompt = this.prompts.current();
    return prompt?.kind === 'quota' ? prompt.usage : null;
  });
  protected readonly failure = computed(() => {
    const prompt = this.prompts.current();
    return prompt?.kind === 'failure' ? prompt.failure : null;
  });

  /** Closed by the person: dismisses the question; closed for the next one: nothing to do. */
  protected onDismissed(event: CustomEvent<{ role?: string }>): void {
    if (event.detail.role === BACKDROP_ROLE) this.prompts.dismiss();
  }
}
