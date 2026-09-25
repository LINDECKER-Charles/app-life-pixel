import { inject, Injectable } from '@angular/core';
import { AlertController } from '@ionic/angular';
import { TranslocoService } from '@jsverse/transloco';

const DISCARD_ROLE = 'destructive';

/** Asks whether the unsaved work may be lost: nothing is kept once it is replaced (D37). */
@Injectable({ providedIn: 'root' })
export class DiscardConfirmation {
  private readonly alerts = inject(AlertController);
  private readonly transloco = inject(TranslocoService);

  /** Resolves to `true` only when the person chooses to discard. */
  async confirm(): Promise<boolean> {
    const translate = (key: string): string => this.transloco.translate(key);
    const alert = await this.alerts.create({
      header: translate('editor.new.discard.title'),
      message: translate('editor.new.discard.message'),
      buttons: [
        { text: translate('common.cancel'), role: 'cancel' },
        { text: translate('editor.new.discard.confirm'), role: DISCARD_ROLE },
      ],
    });
    await alert.present();
    const { role } = await alert.onDidDismiss();
    return role === DISCARD_ROLE;
  }
}
