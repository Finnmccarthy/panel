import { Ref, useCallback, useRef, useState } from 'react';
import { z } from 'zod';
import getNodeTransferringServers from '@/api/admin/nodes/transfers/getNodeTransferringServers.ts';
import AdminSubContentContainer from '@/elements/containers/AdminSubContentContainer.tsx';
import SelectionArea from '@/elements/SelectionArea.tsx';
import Table from '@/elements/Table.tsx';
import { queryKeys } from '@/lib/queryKeys.ts';
import { adminNodeSchema, adminNodeTransfersSchema } from '@/lib/schemas/admin/nodes.ts';
import { useSearchablePaginatedTable } from '@/plugins/useSearchablePaginatedTable.ts';
import { useWebsocket } from '@/plugins/useWebsocket.ts';
import { useToast } from '@/providers/ToastProvider.tsx';
import { useTranslations } from '@/providers/TranslationProvider.tsx';
import ServerRow, { TransferProgressWithRates } from './ServerRow.tsx';

export default function AdminNodeTransfers({ node }: { node: z.infer<typeof adminNodeSchema> }) {
  const { t } = useTranslations();
  const { addToast } = useToast();

  const {
    data: nodeTransferringServers,
    loading,
    error,
    search,
    setSearch,
    setPage,
    refetch,
  } = useSearchablePaginatedTable({
    queryKey: queryKeys.admin.nodes.transfers(node.uuid),
    fetcher: (page, search) => getNodeTransferringServers(node.uuid, page, search),
    paginationKey: 'servers',
  });

  const [progress, setProgress] = useState<Record<string, TransferProgressWithRates>>({});
  const lastFrame = useRef<{ at: number; keys: string } | null>(null);
  const staleAfterLoss = useRef(false);

  // Rates are derived where frames arrive rather than during render: rows re-render for
  // unrelated reasons (search input, StrictMode) and a render-time diff would count those as
  // elapsed time. The key set doubles as the completion signal - a server leaves the map once
  // wings stops transferring it - so a changed key set means the paginated list is stale,
  // whether or not the affected server sits on the current page.
  const onFrame = useCallback(
    (frame: z.infer<typeof adminNodeTransfersSchema>) => {
      const now = performance.now();
      const previous = lastFrame.current;
      const elapsedSeconds = previous ? (now - previous.at) / 1000 : 0;

      setProgress((current) =>
        Object.fromEntries(
          Object.entries(frame).map(([uuid, next]) => {
            const last = current[uuid];
            if (!last || elapsedSeconds <= 0) {
              return [uuid, { ...next, archiveRate: last?.archiveRate ?? 0, networkRate: last?.networkRate ?? 0 }];
            }

            return [
              uuid,
              {
                ...next,
                archiveRate: (next.archiveBytesProcessed - last.archiveBytesProcessed) / elapsedSeconds,
                networkRate: (next.networkBytesProcessed - last.networkBytesProcessed) / elapsedSeconds,
              },
            ];
          }),
        ),
      );

      // Transfers can finish while the socket is down, so the first frame back always refetches
      // rather than waiting for the next key change.
      const keys = Object.keys(frame).sort().join(',');
      if (staleAfterLoss.current || (previous && previous.keys !== keys)) {
        staleAfterLoss.current = false;
        refetch();
      }

      lastFrame.current = { at: now, keys };
    },
    [refetch],
  );

  useWebsocket({
    path: `/api/admin/nodes/${node.uuid}/transfers/ws`,
    schema: adminNodeTransfersSchema,
    reconnectDelay: 5000,
    onMessage: onFrame,
    onConnectionLost: () => {
      lastFrame.current = null;
      staleAfterLoss.current = true;
      setProgress({});
      addToast(t('pages.admin.nodes.tabs.transfers.page.toast.connectionLost', {}), 'error');
    },
  });

  return (
    <>
      <AdminSubContentContainer
        title={t('pages.admin.nodes.tabs.transfers.page.title', {})}
        titleOrder={2}
        search={search}
        setSearch={setSearch}
        registry={window.extensionContext.extensionRegistry.pages.admin.nodes.view.transfers.subContainer}
        registryProps={{ node }}
      >
        <Table
          columns={[
            t('common.table.columns.id', {}),
            t('pages.admin.nodes.tabs.transfers.page.table.columns.progress', {}),
            t('pages.admin.nodes.tabs.transfers.page.table.columns.archiveRate', {}),
            t('pages.admin.nodes.tabs.transfers.page.table.columns.networkRate', {}),
            t('common.table.columns.name', {}),
            t('common.table.columns.node', {}),
            t('common.table.columns.owner', {}),
            t('common.table.columns.created', {}),
          ]}
          loading={loading}
          error={error}
          pagination={nodeTransferringServers?.servers}
          onPageSelect={setPage}
          allowSelect={false}
        >
          {nodeTransferringServers?.servers.data.map((server) => (
            <SelectionArea.Selectable key={server.uuid} item={server}>
              {(innerRef: Ref<HTMLElement>) => (
                <ServerRow
                  key={server.uuid}
                  server={server}
                  transferProgress={progress[server.uuid]}
                  ref={innerRef as Ref<HTMLTableRowElement>}
                />
              )}
            </SelectionArea.Selectable>
          ))}
        </Table>
      </AdminSubContentContainer>
    </>
  );
}
