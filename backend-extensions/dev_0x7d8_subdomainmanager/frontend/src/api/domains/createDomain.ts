import { z } from 'zod';
import { axiosInstance } from '@/api/axios.ts';
import { transformKeysToSnakeCase } from '@/lib/transformers.ts';
import { domainFormSchema } from '../../lib/schemas.ts';

export default async (form: z.infer<typeof domainFormSchema>): Promise<string> => {
  const { data } = await axiosInstance.post(
    '/api/admin/extensions/dev.0x7d8.subdomainmanager/domains',
    transformKeysToSnakeCase(form),
  );
  return data.uuid;
};
