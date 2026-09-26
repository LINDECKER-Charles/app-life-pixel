import { ChangeDetectionStrategy, Component, inject } from '@angular/core';
import { IonApp, IonRouterOutlet } from '@ionic/angular';
import { AppFooter } from './shell/app-footer';
import { AppHeader } from './shell/app-header';
import { EngineNotifications } from './shell/engine-notifications';
import { SupportScreen } from './support/support-screen';

/**
 * The shell: the header above the routed page, the footer, and the engine's notifications. It
 * starts following the screens a support request names as its context.
 */
@Component({
  imports: [AppFooter, AppHeader, EngineNotifications, IonApp, IonRouterOutlet],
  selector: 'lp-root',
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './app.html',
  styleUrl: './app.scss',
})
export class App {
  constructor() {
    inject(SupportScreen).start();
  }
}
