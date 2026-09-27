import {
  EnvironmentProviders,
  isDevMode,
  makeEnvironmentProviders,
  provideAppInitializer,
} from '@angular/core';
import { provideTransloco, TRANSLOCO_TRANSPILER } from '@jsverse/transloco';
import { CatalogueLoader } from './catalogue-loader';
import { SOURCE_LANGUAGE } from './choose-language';
import { IcuTranspiler } from './icu-transpiler';
import { initializeI18n } from './initialize-i18n';

/**
 * Translates the interface with the catalogues served at `/i18n/`, one language at a time, with
 * ICU messages, formatted without `eval` so that the desktop app's CSP allows them; switching
 * language re-renders without reloading. Needs `provideHttpClient()`.
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
    { provide: TRANSLOCO_TRANSPILER, useClass: IcuTranspiler },
    provideAppInitializer(initializeI18n),
  ]);
}
