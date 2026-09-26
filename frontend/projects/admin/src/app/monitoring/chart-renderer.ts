import { Injectable } from '@angular/core';
import type uPlot from 'uplot';
import type { ChartData } from './chart-data';

/** A drawn chart, to resize or remove. */
export interface ChartHandle {
  resize(width: number): void;
  destroy(): void;
}

/** What to draw: the aligned data, and how the value axis writes a number. */
export interface ChartSpec {
  readonly data: ChartData;
  readonly format: (value: number) => string;
  readonly width: number;
  readonly height: number;
}

/**
 * Draws a chart into an element. The console's is uPlot, loaded with the first chart; the unit
 * tests, which run without a canvas, provide their own.
 */
@Injectable({ providedIn: 'root', useFactory: () => new UPlotRenderer() })
export abstract class ChartRenderer {
  abstract render(target: HTMLElement, spec: ChartSpec): Promise<ChartHandle>;
}

const LINE_COLOURS = 4;
const DASH = [6, 4];

/** A colour of the design tokens, read on the chart's element so that dark mode applies. */
function token(target: HTMLElement, name: string): string {
  return getComputedStyle(target).getPropertyValue(name).trim();
}

function seriesOptions(target: HTMLElement, spec: ChartSpec): uPlot.Series[] {
  let current = 0;
  let previous = 0;
  return spec.data.lines.map((line) => {
    const index = line.previous ? previous++ : current++;
    return {
      label: line.label,
      stroke: token(target, `--lpa-chart-${(index % LINE_COLOURS) + 1}`),
      width: line.previous ? 1.5 : 2,
      dash: line.previous ? DASH : undefined,
      spanGaps: false,
    };
  });
}

function axisOptions(target: HTMLElement, spec: ChartSpec): uPlot.Axis[] {
  const text = token(target, '--lp-color-text-muted');
  const grid = { stroke: token(target, '--lpa-chart-grid'), width: 1 };
  return [
    { stroke: text, grid, ticks: grid },
    {
      stroke: text,
      grid,
      ticks: grid,
      size: 80,
      values: (_chart, values) => values.map((value) => spec.format(value)),
    },
  ];
}

/** The uPlot renderer: one time axis, one value scale, the previous period dashed. */
export class UPlotRenderer extends ChartRenderer {
  async render(target: HTMLElement, spec: ChartSpec): Promise<ChartHandle> {
    const { default: UPlot } = await import('uplot');
    const options: uPlot.Options = {
      width: spec.width,
      height: spec.height,
      legend: { show: false },
      scales: { x: { time: true } },
      series: [{}, ...seriesOptions(target, spec)],
      axes: axisOptions(target, spec),
    };
    const data = [spec.data.times, ...spec.data.lines.map((line) => line.values)];
    const chart = new UPlot(options, data as uPlot.AlignedData, target);
    return {
      resize: (width) => chart.setSize({ width, height: spec.height }),
      destroy: () => chart.destroy(),
    };
  }
}
