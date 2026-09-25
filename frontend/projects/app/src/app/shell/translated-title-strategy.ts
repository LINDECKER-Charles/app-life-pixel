import { inject, Injectable } from '@angular/core';
import { Title } from '@angular/platform-browser';
import { RouterStateSnapshot, TitleStrategy } from '@angular/router';
import { TranslocoService } from '@jsverse/transloco';
import { BehaviorSubject, combineLatest, of, switchMap } from 'rxjs';

const APP_NAME_KEY = 'app.name';
const PAGE_TITLE_KEY = 'shell.page_title';

/**
 * Titles the browser tab after the page: a route's `title` is an i18n key, shown with the app's
 * name and translated again when the language changes.
 */
@Injectable({ providedIn: 'root' })
export class TranslatedTitleStrategy extends TitleStrategy {
  private readonly pageKey = new BehaviorSubject<string | undefined>(undefined);

  constructor() {
    super();
    const title = inject(Title);
    const transloco = inject(TranslocoService);
    this.pageKey
      .pipe(
        switchMap((key) =>
          combineLatest([
            transloco.selectTranslate(APP_NAME_KEY),
            key === undefined ? of(undefined) : transloco.selectTranslate(key),
          ]),
        ),
        switchMap(([app, page]) =>
          page === undefined ? of(app) : transloco.selectTranslate(PAGE_TITLE_KEY, { page, app }),
        ),
      )
      .subscribe((text: string) => title.setTitle(text));
  }

  override updateTitle(snapshot: RouterStateSnapshot): void {
    this.pageKey.next(this.buildTitle(snapshot));
  }
}
