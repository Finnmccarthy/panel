import { z } from 'zod';
import { nullableString } from '@/lib/transformers.ts';

export const typeEggAssignmentSchema = z.object({
  affectedTypes: z.array(z.string().min(1)),
  egg: z.uuid(),
});

export const mcjarsTypeOrderSchema = z.record(z.string(), z.array(z.string().uppercase()));

export const adminExtensionSettingsSchema = z.object({
  mcjarsUrl: z.url(),
  mcjarsApiKey: z.preprocess(nullableString, z.string().length(64).nullable()),
  mcjarsIconBaseUrl: z.url().endsWith('/'),
  mcjarsIconFileExtension: z.string().min(1).max(5),

  mcjarsTypeOrder: mcjarsTypeOrderSchema.nullable(),

  typeEggAssignments: z.array(typeEggAssignmentSchema),

  collectInstallationStatistics: z.boolean(),
});
