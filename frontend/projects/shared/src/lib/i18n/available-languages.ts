import { HttpClient } from '@angular/common/http';
import { inject, Injectable, signal } from '@angular/core';
import { firstValueFrom } from 'rxjs';
import { I18N_PATH } from './i18n-path';
import { Language } from './language';

/** The languages `languages.json` lists, read once at start-up. */
@Injectable({ providedIn: 'root' })
export class AvailableLanguages {
  private readonly http = inject(HttpClient);
  private readonly list = signal<readonly Language[]>([]);

  /** The languages to offer; empty when the list could not be read. */
  readonly languages = this.list.asReadonly();

  /** Reads the list; an unreachable list leaves it empty, and the app in its source language. */
  async load(): Promise<void> {
    try {
      const languages = await firstValueFrom(
        this.http.get<readonly Language[]>(`${I18N_PATH}/languages.json`),
      );
      this.list.set(languages);
    } catch (error: unknown) {
      console.error('The list of languages could not be read.', error);
    }
  }
}
