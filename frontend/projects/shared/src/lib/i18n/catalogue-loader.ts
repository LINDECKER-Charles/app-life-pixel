import { HttpClient } from '@angular/common/http';
import { inject, Injectable } from '@angular/core';
import { Translation, TranslocoLoader } from '@jsverse/transloco';
import { Observable } from 'rxjs';
import { I18N_PATH } from './i18n-path';

/** Fetches one catalogue at a time, when its language becomes active: none is bundled. */
@Injectable({ providedIn: 'root' })
export class CatalogueLoader implements TranslocoLoader {
  private readonly http = inject(HttpClient);

  getTranslation(languageCode: string): Observable<Translation> {
    return this.http.get<Translation>(`${I18N_PATH}/${languageCode}.json`);
  }
}
