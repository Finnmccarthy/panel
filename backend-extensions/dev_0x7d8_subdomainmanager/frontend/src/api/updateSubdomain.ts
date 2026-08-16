import { axiosInstance } from '@/api/axios.ts';

export default async (serverUuid: string, subdomainUuid: string, allocationUuid: string): Promise<void> => {
  await axiosInstance.patch(`/api/client/servers/${serverUuid}/subdomains/${subdomainUuid}`, {
    allocation_uuid: allocationUuid,
  });
};
