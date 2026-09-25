import { HttpClient } from '@angular/common/http';
import { inject, Injectable } from '@angular/core';
import { catchError, firstValueFrom, of } from 'rxjs';

const LEGAL_PATH = '/i18n/legal';
const FALLBACK_LANGUAGE = 'en';

/** Fetches a legal page's markdown text in `language`, falling back to English (H16). */
@Injectable({ providedIn: 'root' })
export class LegalTextLoader {
  private readonly http = inject(HttpClient);

  async load(language: string, page: string): Promise<string> {
    const text = await this.fetch(language, page);
    if (text !== undefined) {
      return text;
    }
    if (language === FALLBACK_LANGUAGE) {
      return '';
    }
    return (await this.fetch(FALLBACK_LANGUAGE, page)) ?? '';
  }

  private fetch(language: string, page: string): Promise<string | undefined> {
    return firstValueFrom(
      this.http
        .get(`${LEGAL_PATH}/${language}/${page}.md`, { responseType: 'text' })
        .pipe(catchError(() => of(undefined))),
    );
  }
}
