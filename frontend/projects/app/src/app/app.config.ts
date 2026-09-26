import { provideHttpClient, withFetch, withInterceptors } from '@angular/common/http';
import {
  ApplicationConfig,
  inject,
  provideAppInitializer,
  provideBrowserGlobalErrorListeners,
} from '@angular/core';
import {
  provideRouter,
  RouteReuseStrategy,
  TitleStrategy,
  withComponentInputBinding,
} from '@angular/router';
import { IonicRouteStrategy, provideIonicAngular } from '@ionic/angular';
import { API_HEADERS, SESSION_EVENTS, provideI18n, sessionInterceptor } from 'shared';
import { AccountMenu } from './account/account-menu';
import { SessionStore } from './account/session-store';
import { routes } from './app.routes';
import { HostedExportObserver } from './events/hosted-export-observer';
import { EXPORT_OBSERVER } from './export/export-observer';
import { LegalLinks } from './legal/legal-links';
import { PreferencesStore } from './settings/preferences-store';
import { provideAppearance } from './settings/provide-appearance';
import { WebPreferencesStore } from './settings/web-preferences-store';
import { ACCOUNT_MENU_SLOT } from './shell/account-menu-slot';
import { FOOTER_SLOT } from './shell/footer-slot';
import { TranslatedTitleStrategy } from './shell/translated-title-strategy';
import { provideUnsavedWorkGuard } from './shell/unsaved-work-guard';

// One provider per line.
export const appConfig: ApplicationConfig = {
  providers: [
    provideBrowserGlobalErrorListeners(),
    provideIonicAngular(),
    provideRouter(routes, withComponentInputBinding()),
    { provide: RouteReuseStrategy, useClass: IonicRouteStrategy },
    { provide: TitleStrategy, useClass: TranslatedTitleStrategy },
    provideHttpClient(withFetch(), withInterceptors([sessionInterceptor])),
    provideI18n(),
    { provide: PreferencesStore, useClass: WebPreferencesStore },
    provideAppearance(),
    provideUnsavedWorkGuard(),
    { provide: FOOTER_SLOT, useValue: LegalLinks },
    { provide: ACCOUNT_MENU_SLOT, useValue: AccountMenu },
    { provide: API_HEADERS, useFactory: () => inject(SessionStore).csrfHeader, multi: true },
    {
      provide: SESSION_EVENTS,
      useFactory: () => {
        const session = inject(SessionStore);
        return {
          onUnauthenticated: () => session.handleUnauthenticated(),
          onUpdateRequired: () => session.handleUpdateRequired(),
        };
      },
      multi: true,
    },
    // The hosted app's start-up load; T2 will guard this one call for the desktop, which never runs it.
    provideAppInitializer(() => inject(SessionStore).load()),
    { provide: EXPORT_OBSERVER, useClass: HostedExportObserver },
  ],
};
