import { inject, Injectable } from '@angular/core';
import { AlertController } from '@ionic/angular';
import { TranslocoService } from '@jsverse/transloco';

/** What to do with unsaved work when its animation changed on disk. */
export type LibraryChangedChoice = 'reload' | 'keep' | 'copy';

const RELOAD_ROLE = 'reload';
const COPY_ROLE = 'copy';

/**
 * Asks what to do when the open animation changed on disk while the editor holds unsaved work
 * (desktop.md, T3): reload it, keep mine, or save mine as a copy. Dismissing keeps the work.
 */
@Injectable({ providedIn: 'root' })
export class LibraryChangedPrompt {
  private readonly alerts = inject(AlertController);
  private readonly transloco = inject(TranslocoService);

  async ask(): Promise<LibraryChangedChoice> {
    const translate = (key: string): string => this.transloco.translate(key);
    const alert = await this.alerts.create({
      header: translate('mcp.library_changed.title'),
      message: translate('mcp.library_changed.message'),
      buttons: [
        { text: translate('mcp.library_changed.keep'), role: 'cancel' },
        { text: translate('mcp.library_changed.copy'), role: COPY_ROLE },
        { text: translate('mcp.library_changed.reload'), role: RELOAD_ROLE },
      ],
    });
    await alert.present();
    const { role } = await alert.onDidDismiss();
    if (role === RELOAD_ROLE) return 'reload';
    if (role === COPY_ROLE) return 'copy';
    return 'keep';
  }
}
