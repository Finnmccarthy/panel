import { z } from 'zod';
import { axiosInstance } from '@/api/axios.ts';
import { parsePaginationFromApi } from '@/lib/api-transform.ts';
import { adminSubdomainEntrySchema } from '../../lib/schemas.ts';

export default async (
  domainUuid: string,
  page: number = 1,
  search?: string,
): Promise<Pagination<z.infer<typeof adminSubdomainEntrySchema>>> => {
  const { data } = await axiosInstance.get(
    `/api/admin/extensions/dev.0x7d8.subdomainmanager/domains/${domainUuid}/subdomains`,
    { params: { page, search } },
  );
  return parsePaginationFromApi(adminSubdomainEntrySchema, data.subdomains);
};
