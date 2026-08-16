import { z } from 'zod';
import { axiosInstance } from '@/api/axios.ts';
import { parseFromApi } from '@/lib/api-transform.ts';

const serverDomainSchema = z.object({ uuid: z.string(), domain: z.string() });

export default async (serverUuid: string): Promise<z.infer<typeof serverDomainSchema>[]> => {
  const { data } = await axiosInstance.get(`/api/client/servers/${serverUuid}/subdomains/domains`);
  return parseFromApi(z.array(serverDomainSchema), data.domains);
};
