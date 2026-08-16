import { z } from 'zod';
import { axiosInstance } from '@/api/axios.ts';
import { parseFromApi } from '@/lib/api-transform.ts';
import { mcjarsTypeOrderSchema } from '../../lib/schemas.ts';

export default async (): Promise<z.infer<typeof mcjarsTypeOrderSchema>> => {
  const { data } = await axiosInstance.get('/api/admin/extensions/dev.0x7d8.minecraftversionchanger/default-order');
  return parseFromApi(mcjarsTypeOrderSchema, data.mcjars_type_order);
};
