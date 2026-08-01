import { ReactNode } from 'react';
import { makeComponentHookable } from 'shared';
import Card from './Card.tsx';

function ChartBlock({
  icon,
  title,
  value,
  legend,
  overlayIcon,
  overlayLabel,
  className,
  children,
}: {
  icon: ReactNode;
  title: string;
  value?: ReactNode;
  legend?: ReactNode;
  overlayIcon?: ReactNode;
  overlayLabel?: string;
  className?: string;
  children: ReactNode;
}) {
  return (
    <Card p={0} className={`relative flex min-w-0 flex-col ${className ?? ''}`}>
      <div className='flex flex-col items-start gap-1 px-4 pt-3 pb-2 sm:flex-row sm:items-center sm:justify-between sm:gap-2'>
        <h3 className='flex min-w-0 items-center gap-2 truncate transition-colors duration-100'>
          {icon} {title}
        </h3>
        {!overlayLabel && value !== undefined && value !== null && (
          <span className='shrink-0 text-sm tabular-nums'>{value}</span>
        )}
        {!overlayLabel && legend && <span className='flex shrink-0 items-center gap-3 text-sm'>{legend}</span>}
      </div>
      <div className='min-h-60 flex-1 px-4 pb-3'>
        {overlayLabel ? (
          <div className='flex h-full flex-col items-center justify-center gap-2 text-(--mantine-color-dimmed)'>
            {overlayIcon}
            <span className='text-sm'>{overlayLabel}</span>
          </div>
        ) : (
          children
        )}
      </div>
    </Card>
  );
}

export default makeComponentHookable(ChartBlock);
