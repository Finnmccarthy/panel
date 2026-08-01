import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { bytesToString } from '@/lib/size.ts';

export const CHART_WINDOW = 20_000;
export const CHART_DELAY = 2_000;
export const CHART_TICK = 1_000;

const CHART_TICKS = 3;
const CHART_SERIES_COLORS = 4;

export type ChartScale = 'decimal' | 'binary';

interface Sample {
  t: number;
  values: (number | null)[];
}

export interface StreamChartSeries {
  key: string;
  label: string;
  color: string;
  value: number | null;
  formatted: string | null;
}

export interface StreamChartProps {
  data: Record<string, number | null>[];
  domain: [number, number];
  ticks: number[];
  yMax: number;
  series: StreamChartSeries[];
  format: (value: number) => string;
}

export interface UseStreamChartOptions {
  series: string[];
  format: (value: number) => string;
  scale?: ChartScale;
  min?: number;
}

export function formatPercent(value: number): string {
  return `${Number(value.toFixed(2))}%`;
}

export function formatBytes(value: number): string {
  return bytesToString(value, 2, true);
}

export function formatBytesRate(value: number): string {
  return `${bytesToString(value, 2, true)}/s`;
}

function niceCeil(value: number, scale: ChartScale): number {
  if (!Number.isFinite(value) || value <= 0) {
    return scale === 'binary' ? 1024 : 1;
  }

  if (scale === 'binary') {
    return 2 ** Math.ceil(Math.log2(value));
  }

  const magnitude = 10 ** Math.floor(Math.log10(value));
  return ([1, 2, 4, 5, 10].find((step) => magnitude * step >= value) ?? 10) * magnitude;
}

function seriesColor(index: number): string {
  return `var(--chart-series-${(index % CHART_SERIES_COLORS) + 1})`;
}

export function useStreamChart({ series: labels, format, scale = 'decimal', min = 0 }: UseStreamChartOptions) {
  const samples = useRef<Sample[]>([]);
  const ceiling = useRef(0);
  const [end, setEnd] = useState(() => Date.now() - CHART_DELAY);

  useEffect(() => {
    const interval = setInterval(() => setEnd(Date.now() - CHART_DELAY), CHART_TICK);

    return () => clearInterval(interval);
  }, []);

  const push = useCallback((values: number | null | (number | null)[]) => {
    const now = Date.now();

    samples.current.push({ t: now, values: Array.isArray(values) ? values : [values] });

    const oldest = now - (CHART_WINDOW + CHART_DELAY + 4 * CHART_TICK);
    while (samples.current.length > 0 && samples.current[0].t < oldest) {
      samples.current.shift();
    }
  }, []);

  const clear = useCallback(() => {
    samples.current = [];
    setEnd(Date.now() - CHART_DELAY);
  }, []);

  const { data, ticks, yMax, values } = useMemo(() => {
    const start = end - CHART_WINDOW;
    const visible = samples.current.filter((sample) => sample.t >= start - 2 * CHART_TICK);

    let peak = min;
    for (const sample of visible) {
      for (const value of sample.values) {
        if (value !== null && value > peak) {
          peak = value;
        }
      }
    }

    const wanted = niceCeil(peak * 1.25, scale);
    if (wanted > ceiling.current || wanted <= ceiling.current / 2) {
      ceiling.current = wanted;
    }

    const height = ceiling.current;

    return {
      data: visible.map((sample) => {
        const row: Record<string, number | null> = { t: sample.t };
        for (let i = 0; i < labels.length; i++) {
          row[`v${i}`] = sample.values[i] ?? null;
        }
        return row;
      }),
      ticks: Array.from({ length: CHART_TICKS }, (_, i) => (height * i) / (CHART_TICKS - 1)),
      yMax: height,
      values: samples.current.at(-1)?.values ?? [],
    };
  }, [end, labels, min, scale]);

  const series = useMemo<StreamChartSeries[]>(
    () =>
      labels.map((label, index) => {
        const value = values[index] ?? null;

        return {
          key: `v${index}`,
          label,
          color: seriesColor(index),
          value,
          formatted: value === null ? null : format(value),
        };
      }),
    [labels, values, format],
  );

  return {
    props: {
      data,
      domain: [end - CHART_WINDOW, end] as [number, number],
      ticks,
      yMax,
      series,
      format,
    } satisfies StreamChartProps,
    series,
    value: series.length === 1 ? series[0].formatted : null,
    push,
    clear,
  };
}
