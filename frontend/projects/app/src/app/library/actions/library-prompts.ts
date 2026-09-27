import { inject, Injectable } from '@angular/core';
import { AlertController, type AlertInput } from '@ionic/angular';
import { TranslocoService } from '@jsverse/transloco';
import type { Project } from '../library-types';

const CONFIRM_ROLE = 'confirm';
const DESTROY_ROLE = 'destructive';
const NAME_INPUT = 'name';

/** A question for a name: its heading, the field's label, and the name to start from. */
export interface NameQuestion {
  readonly titleKey: string;
  readonly labelKey: string;
  readonly value: string;
}

/** A confirmation before destroying something: its heading, its message, and what it names. */
export interface DestroyQuestion {
  readonly titleKey: string;
  readonly messageKey: string;
  readonly name: string;
}

/** The value an alert's dismissal carries, when it carries its inputs' values. */
function valuesOf(data: unknown): Record<string, unknown> {
  if (typeof data !== 'object' || data === null) return {};
  const { values } = data as { values?: unknown };
  return typeof values === 'object' && values !== null ? (values as Record<string, unknown>) : {};
}

/**
 * The library pages' short questions, as Ionic alerts: a name to give, a project to move to, and
 * a confirmation before anything is destroyed (accounts.md, H8).
 */
@Injectable({ providedIn: 'root' })
export class LibraryPrompts {
  private readonly alerts = inject(AlertController);
  private readonly transloco = inject(TranslocoService);

  /** The name typed, trimmed, or `null` when cancelled or left empty. */
  async askName(question: NameQuestion): Promise<string | null> {
    const label = this.translate(question.labelKey);
    const input: AlertInput = {
      name: NAME_INPUT,
      type: 'text',
      value: question.value,
      placeholder: label,
      attributes: { 'aria-label': label, required: true },
    };
    const values = await this.ask(question.titleKey, [input], 'library.rename');
    if (values === null || typeof values === 'string') return null;
    const name = values[NAME_INPUT];
    return typeof name === 'string' && name.trim() !== '' ? name.trim() : null;
  }

  /** The project picked among `projects`, or `null` when cancelled. */
  async pickProject(projects: readonly Project[], currentId: string): Promise<string | null> {
    const inputs = projects.map<AlertInput>((project) => ({
      type: 'radio',
      label: project.name,
      value: project.id,
      checked: project.id === currentId,
    }));
    const values = await this.ask('library.animation.move_title', inputs, 'library.move');
    return typeof values === 'string' ? values : null;
  }

  /** Resolves to `true` only when the person confirms the destruction. */
  async confirmDestroy(question: DestroyQuestion): Promise<boolean> {
    const alert = await this.alerts.create({
      header: this.translate(question.titleKey),
      message: this.transloco.translate(question.messageKey, { name: question.name }),
      buttons: [
        { text: this.translate('common.cancel'), role: 'cancel' },
        { text: this.translate('library.delete'), role: DESTROY_ROLE },
      ],
    });
    await alert.present();
    const { role } = await alert.onDidDismiss();
    return role === DESTROY_ROLE;
  }

  /** The values of `inputs` once confirmed, or `null` when cancelled. */
  private async ask(
    titleKey: string,
    inputs: AlertInput[],
    confirmKey: string,
  ): Promise<Record<string, unknown> | string | null> {
    const alert = await this.alerts.create({
      header: this.translate(titleKey),
      inputs,
      buttons: [
        { text: this.translate('common.cancel'), role: 'cancel' },
        { text: this.translate(confirmKey), role: CONFIRM_ROLE },
      ],
    });
    await alert.present();
    const { role, data } = await alert.onDidDismiss<unknown>();
    if (role !== CONFIRM_ROLE) return null;
    const { values } = (data ?? {}) as { values?: unknown };
    return typeof values === 'string' ? values : valuesOf(data);
  }

  private translate(key: string): string {
    return this.transloco.translate(key);
  }
}
