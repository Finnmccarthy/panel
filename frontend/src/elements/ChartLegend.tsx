import { makeComponentHookable } from 'shared';
import { StreamChartSeries } from '@/lib/chart.ts';

function ChartLegend({ series }: { series: StreamChartSeries[] }) {
  return (
    <>
      {series.map((entry) => (
        <span key={entry.key} className='flex items-center gap-1.5 text-xs'>
          <span className='size-2 shrink-0 rounded-full' style={{ backgroundColor: entry.color }} />
          {entry.label}
          {entry.formatted !== null && (
            <span className='tabular-nums text-(--mantine-color-dimmed)'>{entry.formatted}</span>
          )}
        </span>
      ))}
    </>
  );
}

export default makeComponentHookable(ChartLegend);
