import { provideHttpClient } from '@angular/common/http';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';
import { LegalTextLoader } from './legal-text-loader';

async function answer(url: string, body: string, status = 200): Promise<void> {
  const http = TestBed.inject(HttpTestingController);
  const request = await vi.waitFor(() => http.expectOne(url));
  if (status === 200) {
    request.flush(body);
  } else {
    request.flush(body, { status, statusText: 'Not Found' });
  }
}

describe('LegalTextLoader', () => {
  function setup(): LegalTextLoader {
    TestBed.configureTestingModule({
      providers: [provideHttpClient(), provideHttpClientTesting()],
    });
    return TestBed.inject(LegalTextLoader);
  }

  afterEach(() => TestBed.inject(HttpTestingController).verify());

  it("fetches the requested language's markdown", async () => {
    const loader = setup();
    const promise = loader.load('fr', 'terms');
    await answer('/i18n/legal/fr/terms.md', '# Conditions');

    expect(await promise).toBe('# Conditions');
  });

  it('falls back to English when the language has no such page', async () => {
    const loader = setup();
    const promise = loader.load('de', 'privacy');
    await answer('/i18n/legal/de/privacy.md', '', 404);
    await answer('/i18n/legal/en/privacy.md', '# Privacy');

    expect(await promise).toBe('# Privacy');
  });

  it('returns an empty text when English itself has no such page', async () => {
    const loader = setup();
    const promise = loader.load('en', 'cookies');
    await answer('/i18n/legal/en/cookies.md', '', 404);

    expect(await promise).toBe('');
  });
});
