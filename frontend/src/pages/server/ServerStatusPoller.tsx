import { useEffect } from 'react';
import getServer from '@/api/server/getServer.ts';
import { queryKeys } from '@/lib/queryKeys.ts';
import { usePollingResource } from '@/plugins/usePollingResource.ts';
import { useServerStore, useServerStoreApi } from '@/stores/server.ts';

const isTransient = (status: string | null | undefined) => !!status && status !== 'install_failed';

export default function ServerStatusPoller() {
  const serverStoreApi = useServerStoreApi();
  const uuid = useServerStore((state) => state.server.uuid);
  const status = useServerStore((state) => state.server.status);
  const updateServer = useServerStore((state) => state.updateServer);

  const { data } = usePollingResource({
    queryKey: queryKeys.server(uuid).detail(),
    queryFn: () => getServer(uuid),
    interval: 15000,
    enabled: !!uuid && isTransient(status),
    silent: true,
    stopWhen: (server) => !isTransient(server.status),
  });

  useEffect(() => {
    if (!data) return;

    const current = serverStoreApi.getState().server;
    if (current.uuid !== data.uuid || !isTransient(current.status)) return;

    updateServer(data);
  }, [data]);

  return null;
}
