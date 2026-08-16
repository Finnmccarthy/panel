import { axiosInstance } from '@/api/axios.ts';

export default async (serverUuid: string, subdomainUuid: string): Promise<void> => {
  await axiosInstance.delete(`/api/client/servers/${serverUuid}/subdomains/${subdomainUuid}`);
};
