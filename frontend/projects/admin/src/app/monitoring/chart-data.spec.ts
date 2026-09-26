import type { PanelSeries } from '../core/admin-types';
import { alignSeries, hasPoints, seriesLabel, summarize } from './chart-data';

const PANEL: PanelSeries = {
  environment: 'production',
  panel: 'latency',
  from: 100,
  to: 400,
  stepSeconds: 100,
  series: [{ name: 'p95', labels: {}, times: [100, 200, 300], values: [0.2, null, 0.4] }],
  previous: [{ name: 'p95', labels: {}, times: [200, 400], values: [0.1, 0.3] }],
};

describe('chart data', () => {
  it('puts a series and the previous period on the union of their times', () => {
    const data = alignSeries(PANEL);

    expect(data.times).toEqual([100, 200, 300, 400]);
    expect(data.lines).toEqual([
      { label: 'p95', previous: false, values: [0.2, null, 0.4, null] },
      { label: 'p95', previous: true, values: [null, 0.1, null, 0.3] },
    ]);
  });

  it('labels a series by its labels, or by its name without any', () => {
    expect(seriesLabel({ name: 'p95', labels: {}, times: [], values: [] })).toBe('p95');
    expect(
      seriesLabel({
        name: 'p95',
        labels: { route: '/api/v1/x', method: 'GET' },
        times: [],
        values: [],
      }),
    ).toBe('/api/v1/x · GET');
  });

  it('knows a chart with no point', () => {
    expect(hasPoints(alignSeries(PANEL))).toBe(true);
    expect(
      hasPoints({ times: [1], lines: [{ label: 'x', previous: false, values: [null] }] }),
    ).toBe(false);
  });

  it('sums a line up: its last, lowest and highest values', () => {
    expect(summarize({ label: 'p95', previous: false, values: [0.2, null, 0.4, null] })).toEqual({
      label: 'p95',
      previous: false,
      last: 0.4,
      min: 0.2,
      max: 0.4,
    });
    expect(summarize({ label: 'x', previous: true, values: [null] })).toMatchObject({
      last: null,
      min: null,
      max: null,
    });
  });
});
