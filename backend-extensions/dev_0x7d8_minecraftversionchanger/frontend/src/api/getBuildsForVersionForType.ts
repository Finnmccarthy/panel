import { axiosInstance } from '@/api/axios';
import { parsePaginationFromApi } from '@/lib/api-transform.ts';
import { MinecraftBuild, minecraftBuildSchema } from './getVersionsForType.ts';

export default async (
  uuid: string,
  type: string,
  version: string,
  page: number,
  search?: string,
): Promise<Pagination<MinecraftBuild>> => {
  const { data } = await axiosInstance.get(
    `/api/client/servers/${uuid}/minecraft/versions/types/${type.toUpperCase()}/${version}`,
    { params: { page, search } },
  );
  return parsePaginationFromApi(minecraftBuildSchema, data.builds);
};
