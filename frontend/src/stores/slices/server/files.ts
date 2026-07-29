import { z } from 'zod';
import { StateCreator } from 'zustand';
import { serverFileOperationSchema } from '@/lib/schemas/server/files.ts';
import { ServerStore } from '@/stores/server.ts';

export const FAILED_OPERATION_LINGER_MS = 5000;

export interface FilesSlice {
  fileOperations: Map<string, z.infer<typeof serverFileOperationSchema>>;
  failedFileOperations: Map<string, number>;

  _failedFileOperationTimeouts: Map<string, ReturnType<typeof setTimeout>>;

  setFileOperation: (uuid: string, operation: z.infer<typeof serverFileOperationSchema>) => void;
  failFileOperation: (uuid: string) => void;
  removeFileOperation: (uuid: string) => void;
}

export const createFilesSlice: StateCreator<ServerStore, [], [], FilesSlice> = (set, get): FilesSlice => ({
  fileOperations: new Map<string, z.infer<typeof serverFileOperationSchema>>(),
  failedFileOperations: new Map<string, number>(),

  _failedFileOperationTimeouts: new Map<string, ReturnType<typeof setTimeout>>(),

  setFileOperation: (uuid, operation) =>
    set((state) => {
      if (state.failedFileOperations.has(uuid)) return state;

      const newMap = new Map(state.fileOperations);
      newMap.set(uuid, operation);
      return { ...state, fileOperations: newMap };
    }),
  failFileOperation: (uuid) => {
    const state = get();
    if (!state.fileOperations.has(uuid) || state.failedFileOperations.has(uuid)) return;

    state._failedFileOperationTimeouts.set(
      uuid,
      setTimeout(() => get().removeFileOperation(uuid), FAILED_OPERATION_LINGER_MS),
    );

    set((s) => {
      const newMap = new Map(s.failedFileOperations);
      newMap.set(uuid, Date.now());
      return { ...s, failedFileOperations: newMap };
    });
  },
  removeFileOperation: (uuid) =>
    set((state) => {
      const timeout = state._failedFileOperationTimeouts.get(uuid);
      if (timeout) {
        clearTimeout(timeout);
        state._failedFileOperationTimeouts.delete(uuid);
      }

      const newMap = new Map(state.fileOperations);
      newMap.delete(uuid);

      if (!state.failedFileOperations.has(uuid)) {
        return { ...state, fileOperations: newMap };
      }

      const newFailedMap = new Map(state.failedFileOperations);
      newFailedMap.delete(uuid);
      return {
        ...state,
        fileOperations: newMap,
        failedFileOperations: newFailedMap,
      };
    }),
});
