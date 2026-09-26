import { inject, Injectable } from '@angular/core';
import { AlertController } from '@ionic/angular';
import { TranslocoService } from '@jsverse/transloco';

const DISCARD_ROLE = 'destructive';
const CANCEL_ROLE = 'cancel';

/**
 * The danger treatment of the button that loses work. Ionic's alert draws every button alike in
 * its Material mode and styles them from its own scoped sheet: inline tokens are the only way in.
 */
const DANGER_BUTTON = {
  style: { color: 'var(--lp-color-danger)', fontWeight: 'var(--lp-font-weight-bold)' },
};

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
        { text: translate('common.cancel'), role: CANCEL_ROLE },
        {
          text: translate('editor.new.discard.confirm'),
          role: DISCARD_ROLE,
          htmlAttributes: DANGER_BUTTON,
        },
      ],
    });
    await alert.present();
    // Nothing keeps the work once replaced: the safe choice holds the focus first, and Enter
    // never discards by accident (design-system/docs/patterns.md, "Destructive actions").
    alert.querySelector<HTMLElement>(`.alert-button-role-${CANCEL_ROLE}`)?.focus();
    const { role } = await alert.onDidDismiss();
    return role === DISCARD_ROLE;
  }
}
