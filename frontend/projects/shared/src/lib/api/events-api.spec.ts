import { provideHttpClient } from '@angular/common/http';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';
import { EventsApi } from './events-api';

function setUp(): { events: EventsApi; http: HttpTestingController } {
  TestBed.configureTestingModule({
    providers: [provideHttpClient(), provideHttpClientTesting()],
  });
  return { events: TestBed.inject(EventsApi), http: TestBed.inject(HttpTestingController) };
}

describe('EventsApi', () => {
  afterEach(() => TestBed.inject(HttpTestingController).verify());

  it('reports a completed export with its format, size and browsing context', async () => {
    const { events, http } = setUp();
    const answer = events.exportCompleted('gif', 1234, {
      platform: 'web',
      appVersion: '0.0.0',
      language: 'fr',
    });
    const request = http.expectOne('/api/v1/events');
    expect(request.request.method).toBe('POST');
    expect(request.request.body).toEqual({
      name: 'export_completed',
      properties: { format: 'gif', bytes: 1234 },
      platform: 'web',
      appVersion: '0.0.0',
      language: 'fr',
    });
    request.flush(null, { status: 202, statusText: 'Accepted' });
    await expect(answer).resolves.toBeUndefined();
  });
});
