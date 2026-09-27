import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { ApplicationInitStatus } from '@angular/core';
import { TestBed } from '@angular/core/testing';
import en from '../../../../../../i18n/en.json';
import languages from '../../../../../../i18n/languages.json';
import { appConfig } from '../app.config';
import { LegalLinks } from './legal-links';

async function answer(url: string, body: object): Promise<void> {
  const http = TestBed.inject(HttpTestingController);
  (await vi.waitFor(() => http.expectOne(url))).flush(body);
}

async function answerAnonymousSession(): Promise<void> {
  const http = TestBed.inject(HttpTestingController);
  (await vi.waitFor(() => http.expectOne('/api/v1/auth/session'))).flush(
    { code: 'auth.unauthenticated', params: {} },
    { status: 401, statusText: 'Unauthorized' },
  );
}

describe('LegalLinks', () => {
  afterEach(() => TestBed.inject(HttpTestingController).verify());

  it('links to the terms, the privacy policy and the legal notice', async () => {
    TestBed.configureTestingModule({
      providers: [...appConfig.providers, provideHttpClientTesting()],
    });
    const initialization = TestBed.inject(ApplicationInitStatus).donePromise;
    await answer('/i18n/languages.json', languages);
    await answer('/i18n/en.json', en);
    await answerAnonymousSession();
    await initialization;

    const fixture = TestBed.createComponent(LegalLinks);
    await fixture.whenStable();

    const links: NodeListOf<HTMLAnchorElement> = fixture.nativeElement.querySelectorAll('a');
    expect([...links].map((link) => link.getAttribute('href'))).toEqual([
      '/legal/terms',
      '/legal/privacy',
      '/legal/notice',
    ]);
    expect([...links].map((link) => link.textContent?.trim())).toEqual([
      en['legal.footer.terms'],
      en['legal.footer.privacy'],
      en['legal.footer.notice'],
    ]);
  });
});
