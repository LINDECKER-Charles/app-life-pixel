import { HttpClient, provideHttpClient, withInterceptors } from '@angular/common/http';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { Provider } from '@angular/core';
import { TestBed } from '@angular/core/testing';
import { firstValueFrom } from 'rxjs';
import { SESSION_EVENTS, type SessionEventHandler } from './session-events';
import { sessionInterceptor } from './session-interceptor';

function handler(events: SessionEventHandler): Provider {
  return { provide: SESSION_EVENTS, useValue: events, multi: true };
}

function setUp(...providers: Provider[]): { http: HttpClient; backend: HttpTestingController } {
  TestBed.configureTestingModule({
    providers: [
      provideHttpClient(withInterceptors([sessionInterceptor])),
      provideHttpClientTesting(),
      ...providers,
    ],
  });
  return { http: TestBed.inject(HttpClient), backend: TestBed.inject(HttpTestingController) };
}

describe('sessionInterceptor', () => {
  afterEach(() => TestBed.inject(HttpTestingController).verify());

  it('tells the handlers of a 401, and still rejects the caller', async () => {
    const onUnauthenticated = vi.fn();
    const { http, backend } = setUp(handler({ onUnauthenticated }));

    const answer = firstValueFrom(http.get('/api/v1/account'));
    backend
      .expectOne('/api/v1/account')
      .flush(
        { code: 'auth.unauthenticated', params: {} },
        { status: 401, statusText: 'Unauthorized' },
      );

    await expect(answer).rejects.toBeTruthy();
    expect(onUnauthenticated).toHaveBeenCalledOnce();
  });

  it('tells the handlers of a 426, and still rejects the caller', async () => {
    const onUpdateRequired = vi.fn();
    const { http, backend } = setUp(handler({ onUpdateRequired }));

    const answer = firstValueFrom(http.get('/api/v1/account'));
    backend
      .expectOne('/api/v1/account')
      .flush(
        { code: 'client.update_required', params: { minimum: '2.0.0' } },
        { status: 426, statusText: 'Upgrade Required' },
      );

    await expect(answer).rejects.toBeTruthy();
    expect(onUpdateRequired).toHaveBeenCalledOnce();
  });

  it('leaves a request that succeeds untouched, and every handler unbothered', async () => {
    const onUnauthenticated = vi.fn();
    const onUpdateRequired = vi.fn();
    const { http, backend } = setUp(handler({ onUnauthenticated, onUpdateRequired }));

    const answer = firstValueFrom(http.get('/healthz'));
    backend.expectOne('/healthz').flush({ status: 'ok' });

    await expect(answer).resolves.toEqual({ status: 'ok' });
    expect(onUnauthenticated).not.toHaveBeenCalled();
    expect(onUpdateRequired).not.toHaveBeenCalled();
  });

  it('works with no handler registered at all', async () => {
    const { http, backend } = setUp();
    const answer = firstValueFrom(http.get('/api/v1/account'));
    backend
      .expectOne('/api/v1/account')
      .flush(
        { code: 'auth.unauthenticated', params: {} },
        { status: 401, statusText: 'Unauthorized' },
      );

    await expect(answer).rejects.toBeTruthy();
  });
});
