import { axiosInstance } from '@/api/axios.ts';

export default async (
  uuid: string,
  instanceUuid: string,
  databaseUuid: string,
  file: File,
  sourceDb: string | null,
  wipe: boolean,
): Promise<void> => {
  await axiosInstance.post(
    `/api/client/servers/${uuid}/databases/instances/${instanceUuid}/databases/${databaseUuid}/import`,
    file,
    {
      params: { source_db: sourceDb ?? undefined, wipe },
      headers: { 'Content-Type': 'application/octet-stream' },
    },
  );
};
