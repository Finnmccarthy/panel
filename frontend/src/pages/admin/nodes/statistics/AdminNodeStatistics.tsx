import {
  faCloudArrowDown,
  faCloudArrowUp,
  faCloudDownload,
  faDatabase,
  faMemory,
  faMicrochip,
  faPen,
  faSearch,
  faUserLarge,
} from '@fortawesome/free-solid-svg-icons';
import { faChartBar } from '@fortawesome/free-solid-svg-icons/faChartBar';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import { useEffect, useState } from 'react';
import { Line } from 'react-chartjs-2';
import { z } from 'zod';
import Card from '@/elements/Card.tsx';
import ChartBlock from '@/elements/ChartBlock.tsx';
import AdminSubContentContainer from '@/elements/containers/AdminSubContentContainer.tsx';
import Group from '@/elements/Group.tsx';
import SemiCircleProgress from '@/elements/SemiCircleProgress.tsx';
import Spinner from '@/elements/Spinner.tsx';
import Title from '@/elements/Title.tsx';
import TitleCard from '@/elements/TitleCard.tsx';
import Tooltip from '@/elements/Tooltip.tsx';
import { useChart, useChartTickLabel } from '@/lib/chart.ts';
import { adminNodeSchema } from '@/lib/schemas/admin/nodes.ts';
import { adminSystemStatisticsSchema } from '@/lib/schemas/admin/system.ts';
import { bytesToString, mapUnitToLocale } from '@/lib/size.ts';
import { useWebsocket } from '@/plugins/useWebsocket.ts';
import { useToast } from '@/providers/ToastProvider.tsx';
import { useTranslations } from '@/providers/TranslationProvider.tsx';

export default function AdminNodeStatistics({ node }: { node: z.infer<typeof adminNodeSchema> }) {
  const { t } = useTranslations();
  const { addToast } = useToast();

  const [stats, setStats] = useState<z.infer<typeof adminSystemStatisticsSchema> | null>(null);

  const cpu = useChartTickLabel(t('pages.admin.nodes.tabs.statistics.page.label.cpu', {}), 100, '%', 2);
  const memory = useChartTickLabel(
    t('pages.admin.nodes.tabs.statistics.page.label.memory', {}),
    stats ? Math.floor(stats.memory.total / 1024 / 1024) : 0,
    mapUnitToLocale('MiB'),
  );
  const disk = useChart(t('pages.admin.nodes.tabs.statistics.page.label.disk', {}), {
    sets: 2,
    options: {
      scales: {
        y: {
          ticks: {
            callback(value) {
              return bytesToString(typeof value === 'string' ? parseInt(value, 10) : value);
            },
          },
        },
      },
    },
    callback(opts, index) {
      return {
        ...opts,
        label: !index
          ? t('pages.admin.nodes.tabs.statistics.page.chart.diskRead', {})
          : t('pages.admin.nodes.tabs.statistics.page.chart.diskWrite', {}),
      };
    },
  });
  const network = useChart(t('pages.admin.nodes.tabs.statistics.page.label.network', {}), {
    sets: 2,
    options: {
      scales: {
        y: {
          ticks: {
            callback(value) {
              return bytesToString(typeof value === 'string' ? parseInt(value, 10) : value);
            },
          },
        },
      },
    },
    callback(opts, index) {
      return {
        ...opts,
        label: !index
          ? t('pages.admin.nodes.tabs.statistics.page.chart.networkInLabel', {})
          : t('pages.admin.nodes.tabs.statistics.page.chart.networkOutLabel', {}),
      };
    },
  });

  useWebsocket({
    path: `/api/admin/nodes/${node.uuid}/system/stats/ws`,
    schema: adminSystemStatisticsSchema,
    reconnectDelay: 5000,
    onMessage: setStats,
    onConnectionLost: () => {
      setStats(null);
      addToast(t('pages.admin.nodes.tabs.statistics.page.toast.connectionLost', {}), 'error');
    },
  });

  useEffect(() => {
    if (!stats) {
      return;
    }

    cpu.push(stats.cpu.used);
    memory.push(Math.floor(stats.memory.used / 1024 / 1024));
    disk.push([stats.disk.readingRate, stats.disk.writingRate]);
    network.push([stats.network.receivingRate, stats.network.sendingRate]);
  }, [stats]);

  return (
    <AdminSubContentContainer
      title={t('pages.admin.nodes.tabs.statistics.page.title', {})}
      titleOrder={2}
      registry={window.extensionContext.extensionRegistry.pages.admin.nodes.view.statistics.subContainer}
      registryProps={{ node }}
    >
      {!stats ? (
        <Spinner.Centered />
      ) : (
        <>
          <div className='mt-4'>
            <TitleCard
              title={t('pages.admin.nodes.tabs.statistics.page.card.resources', {})}
              icon={<FontAwesomeIcon icon={faUserLarge} />}
            >
              <div className='grid grid-cols-1 lg:grid-cols-2 2xl:grid-cols-3 gap-4'>
                <Card>
                  <Group grow>
                    <div className='flex justify-center'>
                      <SemiCircleProgress
                        value={stats.cpu.used}
                        label={<>{stats.cpu.used.toFixed(1)}%</>}
                        filledSegmentColor={stats.cpu.used >= 90 ? 'red' : undefined}
                      />
                    </div>
                    <div className='flex flex-col text-right flex-1'>
                      <Title order={2}>{t('pages.admin.nodes.tabs.statistics.page.label.cpu', {})}</Title>
                      <h2>
                        {t('pages.admin.nodes.tabs.statistics.page.label.cpuThreads', {
                          model: stats.cpu.model,
                          threads: stats.cpu.threads,
                        })}
                      </h2>
                    </div>
                  </Group>
                </Card>
                <Card>
                  <Group grow>
                    <div className='flex justify-center'>
                      <SemiCircleProgress
                        value={(stats.memory.used / stats.memory.total) * 100}
                        label={<>{((stats.memory.used / stats.memory.total) * 100).toFixed(1)}%</>}
                        filledSegmentColor={stats.memory.used / stats.memory.total >= 0.9 ? 'red' : undefined}
                      />
                    </div>
                    <div className='flex flex-col text-right flex-1'>
                      <Title order={2}>{t('pages.admin.nodes.tabs.statistics.page.label.memory', {})}</Title>
                      <h2>
                        {bytesToString(stats.memory.used)} / {bytesToString(stats.memory.total)}
                      </h2>
                      <p className='text-xs'>
                        {t('pages.admin.nodes.tabs.statistics.page.label.usedByWings', {
                          size: bytesToString(stats.memory.usedProcess),
                        })}
                      </p>
                    </div>
                  </Group>
                </Card>
                <Card>
                  <Group grow>
                    <div className='flex justify-center'>
                      <SemiCircleProgress
                        value={(stats.disk.used / stats.disk.total) * 100}
                        label={<>{((stats.disk.used / stats.disk.total) * 100).toFixed(1)}%</>}
                        filledSegmentColor={stats.disk.used / stats.disk.total >= 0.9 ? 'red' : undefined}
                      />
                    </div>
                    <div className='flex flex-col text-right flex-1'>
                      <Title order={2}>{t('pages.admin.nodes.tabs.statistics.page.label.disk', {})}</Title>
                      <h2>
                        {bytesToString(stats.disk.used)} / {bytesToString(stats.disk.total)}
                      </h2>
                    </div>
                  </Group>
                </Card>
                <Card>
                  <Group grow>
                    <div className='flex justify-center'>
                      <SemiCircleProgress value={100} label='--' filledSegmentColor='gray' />
                    </div>
                    <div className='flex flex-col text-right flex-1'>
                      <Title order={2}>{t('pages.admin.nodes.tabs.statistics.page.label.network', {})}</Title>
                      <h2>
                        {t('pages.admin.nodes.tabs.statistics.page.label.networkIn', {
                          in: bytesToString(stats.network.received),
                        })}
                        <br />
                        {t('pages.admin.nodes.tabs.statistics.page.label.networkOut', {
                          out: bytesToString(stats.network.sent),
                        })}
                      </h2>
                    </div>
                  </Group>
                </Card>
              </div>
            </TitleCard>
          </div>
          <div className='mt-4'>
            <TitleCard
              title={t('pages.admin.nodes.tabs.statistics.page.card.graphs', {})}
              icon={<FontAwesomeIcon icon={faChartBar} />}
            >
              <div className='grid grid-cols-1 md:grid-cols-2 gap-4'>
                <ChartBlock
                  icon={<FontAwesomeIcon icon={faMicrochip} />}
                  title={t('pages.admin.nodes.tabs.statistics.page.chart.cpuLoad', {})}
                >
                  <Line {...cpu.props} />
                </ChartBlock>
                <ChartBlock
                  icon={<FontAwesomeIcon icon={faMemory} />}
                  title={t('pages.admin.nodes.tabs.statistics.page.chart.memoryUsage', {})}
                >
                  <Line {...memory.props} />
                </ChartBlock>
                <ChartBlock
                  icon={<FontAwesomeIcon icon={faDatabase} />}
                  title={t('pages.admin.nodes.tabs.statistics.page.chart.diskIo', {})}
                  legend={
                    <>
                      <Tooltip label={t('pages.admin.nodes.tabs.statistics.page.chart.diskRead', {})}>
                        <FontAwesomeIcon icon={faSearch} className='mr-2 h-4 w-4 text-(--chart-series-1-border)' />
                      </Tooltip>
                      <Tooltip label={t('pages.admin.nodes.tabs.statistics.page.chart.diskWrite', {})}>
                        <FontAwesomeIcon icon={faPen} className='h-4 w-4 text-(--chart-series-2-border)' />
                      </Tooltip>
                    </>
                  }
                >
                  <Line {...disk.props} />
                </ChartBlock>
                <ChartBlock
                  icon={<FontAwesomeIcon icon={faCloudDownload} />}
                  title={t('pages.admin.nodes.tabs.statistics.page.chart.networkTraffic', {})}
                  legend={
                    <>
                      <Tooltip label={t('pages.admin.nodes.tabs.statistics.page.chart.outbound', {})}>
                        <FontAwesomeIcon icon={faCloudArrowUp} className='h-4 w-4 text-(--chart-series-1-border)' />
                      </Tooltip>
                      <Tooltip label={t('pages.admin.nodes.tabs.statistics.page.chart.inbound', {})}>
                        <FontAwesomeIcon
                          icon={faCloudArrowDown}
                          className='mr-2 h-4 w-4 text-(--chart-series-2-border)'
                        />
                      </Tooltip>
                    </>
                  }
                >
                  <Line {...network.props} />
                </ChartBlock>
              </div>
            </TitleCard>
          </div>
        </>
      )}
    </AdminSubContentContainer>
  );
}
