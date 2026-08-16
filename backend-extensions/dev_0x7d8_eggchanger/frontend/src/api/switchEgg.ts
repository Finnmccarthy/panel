import { axiosInstance } from '@/api/axios';

interface Data {
  eggUuid: string;

  updateStartup: boolean;
  reinstall: boolean;
  truncateDirectory: boolean;
}

export default async (uuid: string, data: Data): Promise<void> => {
  return new Promise((resolve, reject) => {
    axiosInstance
      .post(`/api/client/servers/${uuid}/settings/egg-changer/switch`, {
        egg_uuid: data.eggUuid,
        update_startup: data.updateStartup,
        reinstall: data.reinstall,
        truncate_directory: data.truncateDirectory,
      })
      .then(() => resolve())
      .catch(reject);
  });
};
