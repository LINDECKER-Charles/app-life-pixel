import { inject, Injectable } from '@angular/core';
import { AlertController } from '@ionic/angular';
import { TranslocoService } from '@jsverse/transloco';

const REVOKE_ROLE = 'destructive';

/** Asks whether a token may be revoked: an agent using it loses access at once, for good. */
@Injectable({ providedIn: 'root' })
export class TokenRevocation {
  private readonly alerts = inject(AlertController);
  private readonly transloco = inject(TranslocoService);

  /** Resolves to `true` only when the person chooses to revoke the token named `name`. */
  async confirm(name: string): Promise<boolean> {
    const translate = (key: string): string => this.transloco.translate(key, { name });
    const alert = await this.alerts.create({
      header: translate('tokens.revoke.title'),
      message: translate('tokens.revoke.message'),
      buttons: [
        { text: translate('common.cancel'), role: 'cancel' },
        { text: translate('tokens.revoke.confirm'), role: REVOKE_ROLE },
      ],
    });
    await alert.present();
    const { role } = await alert.onDidDismiss();
    return role === REVOKE_ROLE;
  }
}
