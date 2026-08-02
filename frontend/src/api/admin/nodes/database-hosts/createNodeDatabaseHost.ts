import { axiosInstance } from '@/api/axios.ts';

export default async (nodeUuid: string, hostUuid: string): Promise<void> => {
  await axiosInstance.post(`/api/admin/nodes/${nodeUuid}/database-hosts`, {
    database_host_uuid: hostUuid,
  });
};
