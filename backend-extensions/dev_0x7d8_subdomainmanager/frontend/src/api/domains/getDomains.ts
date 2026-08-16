import { z } from 'zod';
import { axiosInstance } from '@/api/axios.ts';
import { parsePaginationFromApi } from '@/lib/api-transform.ts';
import { domainSchema } from '../../lib/schemas.ts';

export default async (): Promise<z.infer<typeof domainSchema>[]> => {
  const { data } = await axiosInstance.get('/api/admin/extensions/dev.0x7d8.subdomainmanager/domains');
  return parsePaginationFromApi(domainSchema, data.domains).data;
};
