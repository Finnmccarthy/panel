import { z } from 'zod';
import { axiosInstance } from '@/api/axios.ts';
import { parseFromApi } from '@/lib/api-transform.ts';
import { adminNodeTransfersSchema } from '@/lib/schemas/admin/nodes.ts';

export default async (nodeUuid: string): Promise<z.infer<typeof adminNodeTransfersSchema>> => {
  const { data } = await axiosInstance.get(`/api/admin/nodes/${nodeUuid}/transfers`);
  return parseFromApi(adminNodeTransfersSchema, data.transfers);
};
