import { ChangeDetectionStrategy, Component } from '@angular/core';
import { IonApp, IonRouterOutlet } from '@ionic/angular';
import { AppFooter } from './shell/app-footer';
import { AppHeader } from './shell/app-header';
import { EngineNotifications } from './shell/engine-notifications';

/** The shell: the header above the routed page, the footer, and the engine's notifications. */
@Component({
  imports: [AppFooter, AppHeader, EngineNotifications, IonApp, IonRouterOutlet],
  selector: 'lp-root',
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './app.html',
  styleUrl: './app.scss',
})
export class App {}
