import { faCog, IconDefinition } from '@fortawesome/free-solid-svg-icons';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { Popover } from '@mantine/core';
import { ReactNode } from 'react';
import Button from '@/elements/Button.tsx';
import Card from '@/elements/Card.tsx';
import CopyOnClick from '@/elements/CopyOnClick.tsx';
import ScrollingText from '@/elements/ScrollingText.tsx';
import ThemeIcon from '@/elements/ThemeIcon.tsx';
import { usageColor } from '@/lib/usage.ts';

export default function StatCard({
  icon,
  label,
  value,
  order,
  copyOnClick,
  popover,
  limit,
  details,
  progress,
  total,
}: {
  icon: IconDefinition;
  label: string;
  value: string;
  order?: number;
  copyOnClick?: boolean;
  popover?: ReactNode;
  limit?: string | null;
  details?: string | null;
  progress?: number | null;
  total?: number | null;
}) {
  const color = usageColor(progress, total);

  return (
    <Card style={{ order }} progress={progress} total={total} progressColor={color}>
      <div className='flex flex-row items-center'>
        <ThemeIcon size='xl' radius='md' color={color}>
          <FontAwesomeIcon size='xl' icon={icon} />
        </ThemeIcon>
        <div className='flex flex-col ml-4 w-full min-w-0'>
          <div className='w-full flex justify-between'>
            <span className='text-sm text-left text-(--mantine-color-dimmed) font-bold'>{label}</span>
            {popover && (
              <Popover position='bottom' withArrow shadow='md'>
                <Popover.Target>
                  <Button variant='transparent' size='compact-xs'>
                    <FontAwesomeIcon size='lg' icon={faCog} />
                  </Button>
                </Popover.Target>
                <Popover.Dropdown>{popover}</Popover.Dropdown>
              </Popover>
            )}
          </div>
          <span className='text-lg font-bold max-w-full'>
            {copyOnClick ? (
              <ScrollingText>
                <CopyOnClick content={value} className='text-left block'>
                  {value} {limit && <span className='text-sm text-(--mantine-color-dimmed)'>/ {limit}</span>}{' '}
                  {details && <span className='text-sm text-(--mantine-color-dimmed)'>({details})</span>}
                </CopyOnClick>
              </ScrollingText>
            ) : (
              <ScrollingText>
                {value} {limit && <span className='text-sm text-(--mantine-color-dimmed)'>/ {limit}</span>}{' '}
                {details && <span className='text-sm text-(--mantine-color-dimmed)'>({details})</span>}
              </ScrollingText>
            )}
          </span>
        </div>
      </div>
    </Card>
  );
}
