import { axiosInstance } from '@/api/axios.ts';

export default async (
  serverUuid: string,
  subdomain: string,
  domainUuid: string,
  allocationUuid: string,
): Promise<void> => {
  await axiosInstance.post(`/api/client/servers/${serverUuid}/subdomains`, {
    subdomain,
    domain_uuid: domainUuid,
    allocation_uuid: allocationUuid,
  });
};
