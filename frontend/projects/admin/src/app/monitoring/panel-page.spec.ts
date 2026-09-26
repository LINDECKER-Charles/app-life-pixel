import { ComponentFixture, TestBed } from '@angular/core/testing';
import { ApiProblem } from 'shared';
import { vi } from 'vitest';
import type { PanelSeries } from '../core/admin-types';
import { FileSaver } from '../core/file-saver';
import { PANEL_SERIES } from '../testing/fixtures';
import { openPage, seriousViolations, settled, textOf } from '../testing/admin-test-support';
import { ChartRenderer } from './chart-renderer';
import { MonitoringApi } from './monitoring-api';
import { PanelPage } from './panel-page';

async function openPanel(
  panel: string,
  answer: () => Promise<PanelSeries> = () => Promise.resolve(PANEL_SERIES),
): Promise<ComponentFixture<PanelPage>> {
  const api = { series: vi.fn(answer) };
  return openPage(PanelPage, {
    url: '/monitoring/latency?env=production&from=now-6h',
    inputs: { panel },
    providers: [{ provide: MonitoringApi, useValue: api }],
  });
}

describe('PanelPage', () => {
  it('titles the chart by the question it answers, beside the previous period', async () => {
    const fixture = await openPanel('latency');
    const page = fixture.nativeElement as HTMLElement;
    await settled(() => expect(page.querySelectorAll('tbody tr').length).toBe(2));

    expect(textOf(page.querySelector('figcaption h2'))).toBe(
      'How long do the slowest 5% of requests take?',
    );
    expect(textOf(page.querySelector('tbody'))).toContain('Previous period');
    expect(textOf(page.querySelector('tbody'))).toContain('300 ms');
    const render = (TestBed.inject(ChartRenderer).render as ReturnType<typeof vi.fn>).mock.calls;
    expect(
      render.at(-1)?.[1].data.lines.map((line: { previous: boolean }) => line.previous),
    ).toEqual([false, true]);
  });

  it('exports its values as CSV, named after the panel and the environment', async () => {
    const fixture = await openPanel('latency');
    const page = fixture.nativeElement as HTMLElement;
    await settled(() => expect(page.querySelectorAll('tbody tr').length).toBe(2));

    const exportButton = Array.from(page.querySelectorAll('button')).find(
      (button) => textOf(button) === 'Export as CSV',
    );
    exportButton?.click();

    const saver = TestBed.inject(FileSaver) as unknown as { saveCsv: ReturnType<typeof vi.fn> };
    expect(saver.saveCsv).toHaveBeenCalledWith(expect.any(Array), expect.any(Array), {
      table: 'panel-latency',
      environment: 'production',
    });
  });

  it('says there was nothing over the range', async () => {
    const empty = { ...PANEL_SERIES, series: [], previous: [] };
    const fixture = await openPanel('latency', () => Promise.resolve(empty));

    await settled(() =>
      expect(textOf(fixture.nativeElement.querySelector('[data-reason="nothing"]'))).toBe(
        'No value was recorded over this period.',
      ),
    );
  });

  it('says why the source gave nothing', async () => {
    const fixture = await openPanel('latency', () =>
      Promise.reject(new ApiProblem(503, 'monitoring.unavailable', { source: 'prometheus' })),
    );

    await settled(() =>
      expect(fixture.nativeElement.querySelector('[data-reason="unavailable"]')).not.toBeNull(),
    );
  });

  it('says there is no such panel', async () => {
    const fixture = await openPanel('nope');

    expect(textOf(fixture.nativeElement)).toContain('There is no chart named nope.');
  });

  it('passes axe', async () => {
    const fixture = await openPanel('latency');
    await settled(() => expect(fixture.nativeElement.querySelectorAll('tbody tr').length).toBe(2));

    expect(await seriousViolations(fixture.nativeElement)).toEqual([]);
  });
});
