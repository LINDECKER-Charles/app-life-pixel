import { DOCUMENT, inject } from '@angular/core';
import { TranslocoService } from '@jsverse/transloco';
import { firstValueFrom } from 'rxjs';
import { AvailableLanguages } from './available-languages';
import { chooseLanguage, SOURCE_LANGUAGE } from './choose-language';
import { PREFERRED_LANGUAGE } from './preferred-language';

/**
 * Reads the list of languages, activates the initial one and waits for its catalogue, so that the
 * first render is translated. Runs in an injection context, as an app initializer.
 */
export async function initializeI18n(): Promise<void> {
  const transloco = inject(TranslocoService);
  const availableLanguages = inject(AvailableLanguages);
  const preferredSources = inject(PREFERRED_LANGUAGE, { optional: true }) ?? [];
  const document = inject(DOCUMENT);

  await availableLanguages.load();
  const codes = availableLanguages.languages().map((language) => language.code);
  transloco.setAvailableLangs(codes.length > 0 ? codes : [SOURCE_LANGUAGE]);

  const preferred = await Promise.all(preferredSources.map((source) => source()));
  const browserLanguages = document.defaultView?.navigator.languages ?? [];
  const initial = chooseLanguage([...preferred, ...browserLanguages], codes);

  transloco.langChanges$.subscribe((code) => (document.documentElement.lang = code));
  transloco.setActiveLang(initial);
  await firstValueFrom(transloco.load(initial));
}
