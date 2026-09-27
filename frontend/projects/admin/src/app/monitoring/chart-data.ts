import type { PanelSeries, Series } from '../core/admin-types';

/** One line of a chart: its label, whether it is the previous period's, a value at each time. */
export interface ChartLine {
  readonly label: string;
  readonly previous: boolean;
  readonly values: readonly (number | null)[];
}

/** A panel's series on one time axis, as uPlot draws them: one scale per axis. */
export interface ChartData {
  readonly times: readonly number[];
  readonly lines: readonly ChartLine[];
}

/** What a line's table row says: its last value, its lowest and its highest. */
export interface LineSummary {
  readonly label: string;
  readonly previous: boolean;
  readonly last: number | null;
  readonly min: number | null;
  readonly max: number | null;
}

/** `{ route: "/api/v1/x" }` named `slow_routes`: "/api/v1/x"; `p95` without labels: "p95". */
export function seriesLabel(series: Series): string {
  const labels = Object.values(series.labels).filter((value) => value !== '');
  return labels.length > 0 ? labels.join(' · ') : series.name;
}

function valuesAt(series: Series, times: readonly number[]): (number | null)[] {
  const byTime = new Map(series.times.map((time, index) => [time, series.values[index] ?? null]));
  return times.map((time) => byTime.get(time) ?? null);
}

/**
 * The series and the previous period's — already moved onto the range by the admin server — on
 * the union of their times, a missing point left `null` rather than drawn as zero.
 */
export function alignSeries(panel: PanelSeries): ChartData {
  const all = [...panel.series, ...panel.previous];
  const times = [...new Set(all.flatMap((series) => series.times))].sort((a, b) => a - b);
  const line = (series: Series, previous: boolean): ChartLine => ({
    label: seriesLabel(series),
    previous,
    values: valuesAt(series, times),
  });
  return {
    times,
    lines: [
      ...panel.series.map((series) => line(series, false)),
      ...panel.previous.map((series) => line(series, true)),
    ],
  };
}

/** Whether a chart has anything to draw: a panel with no point says why instead. */
export function hasPoints(data: ChartData): boolean {
  return data.lines.some((line) => line.values.some((value) => value !== null));
}

export function summarize(line: ChartLine): LineSummary {
  const present = line.values.filter((value): value is number => value !== null);
  return {
    label: line.label,
    previous: line.previous,
    last: present.at(-1) ?? null,
    min: present.length > 0 ? Math.min(...present) : null,
    max: present.length > 0 ? Math.max(...present) : null,
  };
}
