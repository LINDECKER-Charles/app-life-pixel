import { provideHttpClient, withFetch } from '@angular/common/http';
import { ApplicationConfig, provideBrowserGlobalErrorListeners } from '@angular/core';
import {
  provideRouter,
  RouteReuseStrategy,
  TitleStrategy,
  withComponentInputBinding,
} from '@angular/router';
import { IonicRouteStrategy, provideIonicAngular } from '@ionic/angular';
import { provideI18n } from 'shared';
import { routes } from './app.routes';
import { PreferencesStore } from './settings/preferences-store';
import { provideAppearance } from './settings/provide-appearance';
import { WebPreferencesStore } from './settings/web-preferences-store';
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
    provideHttpClient(withFetch()),
    provideI18n(),
    { provide: PreferencesStore, useClass: WebPreferencesStore },
    provideAppearance(),
    provideUnsavedWorkGuard(),
  ],
};
