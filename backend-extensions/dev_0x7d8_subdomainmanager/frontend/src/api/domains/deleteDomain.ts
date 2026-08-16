import { axiosInstance } from '@/api/axios.ts';

export default async (uuid: string): Promise<void> => {
  await axiosInstance.delete(`/api/admin/extensions/dev.0x7d8.subdomainmanager/domains/${uuid}`);
};
