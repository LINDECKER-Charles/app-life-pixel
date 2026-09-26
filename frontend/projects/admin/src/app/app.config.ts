import { provideHttpClient, withFetch } from '@angular/common/http';
import {
  ApplicationConfig,
  inject,
  provideAppInitializer,
  provideBrowserGlobalErrorListeners,
} from '@angular/core';
import { provideRouter, TitleStrategy, withComponentInputBinding } from '@angular/router';
import { provideI18n } from 'shared';
import { routes } from './app.routes';
import { SessionStore } from './core/session-store';
import { TranslatedTitleStrategy } from './core/translated-title-strategy';

// One provider per line.
export const appConfig: ApplicationConfig = {
  providers: [
    provideBrowserGlobalErrorListeners(),
    provideRouter(routes, withComponentInputBinding()),
    { provide: TitleStrategy, useClass: TranslatedTitleStrategy },
    provideHttpClient(withFetch()),
    provideI18n(),
    provideAppInitializer(() => inject(SessionStore).load()),
  ],
};
