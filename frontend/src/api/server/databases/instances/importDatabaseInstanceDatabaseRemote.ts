import { z } from 'zod';
import { axiosInstance } from '@/api/axios.ts';
import { serializeForApi } from '@/lib/api-transform.ts';
import { serverDatabaseInstanceRemoteImportSchema } from '@/lib/schemas/server/databaseInstances.ts';

export default async (
  uuid: string,
  instanceUuid: string,
  databaseUuid: string,
  importData: z.infer<typeof serverDatabaseInstanceRemoteImportSchema>,
): Promise<string> => {
  const { data } = await axiosInstance.post(
    `/api/client/servers/${uuid}/databases/instances/${instanceUuid}/databases/${databaseUuid}/import/remote`,
    serializeForApi(serverDatabaseInstanceRemoteImportSchema, importData),
  );
  return data.operation;
};
