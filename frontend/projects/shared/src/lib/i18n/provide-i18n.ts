import {
  EnvironmentProviders,
  isDevMode,
  makeEnvironmentProviders,
  provideAppInitializer,
} from '@angular/core';
import { provideTransloco } from '@jsverse/transloco';
import { provideTranslocoMessageformat } from '@jsverse/transloco-messageformat';
import { CatalogueLoader } from './catalogue-loader';
import { SOURCE_LANGUAGE } from './choose-language';
import { initializeI18n } from './initialize-i18n';

/**
 * Translates the interface with the catalogues served at `/i18n/`, one language at a time, with
 * ICU messages; switching language re-renders without reloading. Needs `provideHttpClient()`.
 */
export function provideI18n(): EnvironmentProviders {
  return makeEnvironmentProviders([
    provideTransloco({
      config: {
        defaultLang: SOURCE_LANGUAGE,
        fallbackLang: SOURCE_LANGUAGE,
        reRenderOnLangChange: true,
        prodMode: !isDevMode(),
        missingHandler: { logMissingKey: true, useFallbackTranslation: false },
      },
      loader: CatalogueLoader,
    }),
    provideTranslocoMessageformat(),
    provideAppInitializer(initializeI18n),
  ]);
}
