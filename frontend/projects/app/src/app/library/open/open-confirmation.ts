import { inject, Injectable } from '@angular/core';
import { AlertController } from '@ionic/angular';
import { TranslocoService } from '@jsverse/transloco';

const DISCARD_ROLE = 'destructive';

/** Asks whether opening another animation may lose the editor's unsaved work (D37). */
@Injectable({ providedIn: 'root' })
export class OpenConfirmation {
  private readonly alerts = inject(AlertController);
  private readonly transloco = inject(TranslocoService);

  /** Resolves to `true` only when the person chooses to open anyway. */
  async confirm(): Promise<boolean> {
    const translate = (key: string): string => this.transloco.translate(key);
    const alert = await this.alerts.create({
      header: translate('library.open.discard.title'),
      message: translate('library.open.discard.message'),
      buttons: [
        { text: translate('common.cancel'), role: 'cancel' },
        { text: translate('library.open.discard.confirm'), role: DISCARD_ROLE },
      ],
    });
    await alert.present();
    const { role } = await alert.onDidDismiss();
    return role === DISCARD_ROLE;
  }
}
