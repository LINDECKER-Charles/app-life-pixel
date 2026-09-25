import { provideHttpClientTesting, HttpTestingController } from '@angular/common/http/testing';
import { ApplicationInitStatus } from '@angular/core';
import { TestBed } from '@angular/core/testing';
import en from '../../../../../i18n/en.json';
import languages from '../../../../../i18n/languages.json';
import { App } from './app';
import { appConfig } from './app.config';

describe('App', () => {
  it('fetches the catalogue of its language at start-up and renders from it', async () => {
    TestBed.configureTestingModule({
      providers: [...appConfig.providers, provideHttpClientTesting()],
    });
    const initialization = TestBed.inject(ApplicationInitStatus).donePromise;
    const http = TestBed.inject(HttpTestingController);
    (await vi.waitFor(() => http.expectOne('/i18n/languages.json'))).flush(languages);
    (await vi.waitFor(() => http.expectOne('/i18n/en.json'))).flush(en);
    await initialization;

    const fixture = TestBed.createComponent(App);
    await fixture.whenStable();

    expect(fixture.nativeElement.querySelector('h1')?.textContent).toBe(en['app.name']);
    http.verify();
  });
});
