import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { ApplicationInitStatus } from '@angular/core';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import { TranslocoService } from '@jsverse/transloco';
import en from '../../../../../../i18n/en.json';
import languages from '../../../../../../i18n/languages.json';
import { appConfig } from '../app.config';
import { LegalPage } from './legal-page';

async function answerJson(url: string, body: object): Promise<void> {
  const http = TestBed.inject(HttpTestingController);
  (await vi.waitFor(() => http.expectOne(url))).flush(body);
}

async function answerText(url: string, body: string, status = 200): Promise<void> {
  const http = TestBed.inject(HttpTestingController);
  const request = await vi.waitFor(() => http.expectOne(url));
  if (status === 200) {
    request.flush(body);
  } else {
    request.flush(body, { status, statusText: 'Not Found' });
  }
}

async function openLegalPage(page: string): Promise<ComponentFixture<LegalPage>> {
  TestBed.configureTestingModule({
    providers: [...appConfig.providers, provideHttpClientTesting()],
  });
  const initialization = TestBed.inject(ApplicationInitStatus).donePromise;
  await answerJson('/i18n/languages.json', languages);
  await answerJson('/i18n/en.json', en);
  await initialization;
  const fixture = TestBed.createComponent(LegalPage);
  fixture.componentRef.setInput('page', page);
  await fixture.whenStable();
  return fixture;
}

function heading(fixture: ComponentFixture<LegalPage>): string | null | undefined {
  fixture.detectChanges();
  const page: HTMLElement | null = fixture.nativeElement.querySelector('.page');
  return page?.querySelector('h1')?.textContent;
}

describe('LegalPage', () => {
  afterEach(() => TestBed.inject(HttpTestingController).verify());

  it("renders the active language's markdown as headings and links", async () => {
    const fixture = await openLegalPage('terms');
    await answerText(
      '/i18n/legal/en/terms.md',
      '# Terms\n\nSee our [privacy policy](/legal/privacy).',
    );
    await vi.waitFor(() => expect(heading(fixture)).toBe('Terms'));

    const page: HTMLElement = fixture.nativeElement.querySelector('.page');
    expect(page.querySelector('a')?.getAttribute('href')).toBe('/legal/privacy');
  });

  it('falls back to English when the active language has no such legal page', async () => {
    const fixture = await openLegalPage('notice');
    await answerText('/i18n/legal/en/notice.md', '# Notice');
    await vi.waitFor(() => expect(heading(fixture)).toBe('Notice'));

    TestBed.inject(TranslocoService).setActiveLang('fr');
    await answerText('/i18n/legal/fr/notice.md', '', 404);
    await answerText('/i18n/legal/en/notice.md', '# Avis (English fallback)');
    await vi.waitFor(() => expect(heading(fixture)).toBe('Avis (English fallback)'));
  });
});
