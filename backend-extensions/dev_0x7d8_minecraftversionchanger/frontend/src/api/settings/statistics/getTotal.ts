import { z } from 'zod';
import { axiosInstance } from '@/api/axios.ts';
import { parseFromApi } from '@/lib/api-transform.ts';

const totalStatisticsSchema = z.object({
  totalInstallations: z.number(),
  installationsByType: z.record(z.string(), z.number()),
  installationsByVersion: z.record(z.string(), z.number()),
});

export type TotalStatistics = z.infer<typeof totalStatisticsSchema>;

export default async (): Promise<TotalStatistics> => {
  const { data } = await axiosInstance.get('/api/admin/extensions/dev.0x7d8.minecraftversionchanger/statistics/total');
  return parseFromApi(totalStatisticsSchema, data);
};
