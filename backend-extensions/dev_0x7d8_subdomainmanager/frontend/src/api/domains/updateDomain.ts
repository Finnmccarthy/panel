import { z } from 'zod';
import { axiosInstance } from '@/api/axios.ts';
import { transformKeysToSnakeCase } from '@/lib/transformers.ts';
import { domainFormSchema } from '../../lib/schemas.ts';

export default async (uuid: string, form: Partial<z.infer<typeof domainFormSchema>>): Promise<void> => {
  await axiosInstance.patch(
    `/api/admin/extensions/dev.0x7d8.subdomainmanager/domains/${uuid}`,
    transformKeysToSnakeCase(form),
  );
};
