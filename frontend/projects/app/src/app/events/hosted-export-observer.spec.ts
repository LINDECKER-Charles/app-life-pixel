import { TestBed } from '@angular/core/testing';
import { TranslocoService } from '@jsverse/transloco';
import { EventsApi, LIFE_PIXEL_CLIENT } from 'shared';
import { HostedExportObserver } from './hosted-export-observer';

function configure(client: string, language: string): { events: EventsApi } {
  const events = { exportCompleted: vi.fn().mockResolvedValue(undefined) } as unknown as EventsApi;
  const transloco = { getActiveLang: () => language } as unknown as TranslocoService;
  TestBed.configureTestingModule({
    providers: [
      { provide: EventsApi, useValue: events },
      { provide: LIFE_PIXEL_CLIENT, useValue: client },
      { provide: TranslocoService, useValue: transloco },
    ],
  });
  return { events };
}

describe('HostedExportObserver', () => {
  it('reports the download through EventsApi, with the client and the active language', () => {
    const { events } = configure('web/0.0.0', 'fr');
    const observer = TestBed.inject(HostedExportObserver);

    observer.record('gif', 1234);

    expect(events.exportCompleted).toHaveBeenCalledWith('gif', 1234, {
      platform: 'web',
      appVersion: '0.0.0',
      language: 'fr',
    });
  });

  it('does not wait for the request before returning', () => {
    const { events } = configure('desktop/1.2.3', 'en');
    const observer = TestBed.inject(HostedExportObserver);

    expect(() => observer.record('wasm', 42)).not.toThrow();
    expect(events.exportCompleted).toHaveBeenCalledWith('wasm', 42, {
      platform: 'desktop',
      appVersion: '1.2.3',
      language: 'en',
    });
  });
});
