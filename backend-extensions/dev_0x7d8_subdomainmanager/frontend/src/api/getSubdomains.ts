import { z } from 'zod';
import { axiosInstance } from '@/api/axios.ts';
import { parsePaginationFromApi } from '@/lib/api-transform.ts';
import { subdomainEntrySchema } from '../lib/schemas.ts';

export default async (
  serverUuid: string,
  page: number = 1,
  search?: string,
): Promise<Pagination<z.infer<typeof subdomainEntrySchema>>> => {
  const { data } = await axiosInstance.get(`/api/client/servers/${serverUuid}/subdomains`, {
    params: { page, search },
  });
  return parsePaginationFromApi(subdomainEntrySchema, data.subdomains);
};
