import { TestBed } from '@angular/core/testing';
import { vi } from 'vitest';
import { FileSaver } from '../core/file-saver';
import { ALERT } from '../testing/fixtures';
import { openPage, seriousViolations, settled, textOf } from '../testing/admin-test-support';
import { AlertsPage } from './alerts-page';
import { MonitoringApi } from './monitoring-api';

async function openAlerts(alerts: unknown[]): Promise<HTMLElement> {
  const fixture = await openPage(AlertsPage, {
    url: '/alerts?env=production',
    providers: [
      { provide: MonitoringApi, useValue: { alerts: vi.fn().mockResolvedValue(alerts) } },
    ],
  });
  await settled(() => expect(textOf(fixture.nativeElement)).not.toContain('Loading'));
  return fixture.nativeElement;
}

describe('AlertsPage', () => {
  it('lists the alerts firing in the environment, and exports them', async () => {
    const page = await openAlerts([ALERT]);

    expect(textOf(page.querySelector('tbody'))).toContain('HighErrorRate');
    Array.from(page.querySelectorAll('button'))
      .find((button) => textOf(button) === 'Export as CSV')
      ?.click();
    const saver = TestBed.inject(FileSaver) as unknown as { saveCsv: ReturnType<typeof vi.fn> };
    expect(saver.saveCsv).toHaveBeenCalledWith([ALERT], expect.any(Array), {
      table: 'alerts',
      environment: 'production',
    });
  });

  it('says no alert is firing, and where', async () => {
    const page = await openAlerts([]);

    expect(textOf(page.querySelector('[data-reason="nothing"]'))).toBe(
      'No alert is firing in production.',
    );
  });

  it('passes axe', async () => {
    const page = await openAlerts([ALERT]);

    expect(await seriousViolations(page)).toEqual([]);
  });
});
