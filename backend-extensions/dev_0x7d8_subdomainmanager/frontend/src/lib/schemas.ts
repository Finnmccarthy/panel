import { z } from 'zod';
import { serverAllocationSchema } from '@/lib/schemas/server/allocations';

export const serverWithSubdomainsFeatureLimitsSchema = z.object({
  featureLimits: z.object({
    subdomains: z.number().int().min(0),
  }),
});

export const domainSchema = z.object({
  uuid: z.string(),
  provider: z.literal('cloudflare'),
  eggs: z.array(z.string()),
  domain: z.string(),
  disallowedSubdomainsRegexes: z.array(z.string()),
  providerConfig: z.record(z.string(), z.unknown()),
});

export const domainFormSchema = z.object({
  provider: z.literal('cloudflare'),
  eggs: z.array(z.string()),
  domain: z.string().min(1).max(255),
  disallowedSubdomainsRegexes: z.array(z.string()),
  providerConfig: z.record(z.string(), z.unknown()),
});

const isValidRegex = (s: string) => {
  try {
    new RegExp(s);
    return true;
  } catch {
    return false;
  }
};
const isValidJson = (s: string) => {
  try {
    JSON.parse(s);
    return true;
  } catch {
    return false;
  }
};

export const domainCreateUpdateFormSchema = z.object({
  provider: z.literal('cloudflare'),
  eggs: z.array(z.string()),
  domain: z.string().min(1).max(255),
  disallowedSubdomainsRegexes: z.array(z.string().refine(isValidRegex, { message: 'Invalid regex pattern' })),
  cloudflare_token: z.string().min(1, { message: 'Token is required' }),
  cloudflare_zone_id: z.string().min(1, { message: 'Zone ID is required' }),
  cloudflare_api_data: z.string().refine(isValidJson, { message: 'Must be valid JSON' }),
});

export const subdomainEntrySchema = z.object({
  uuid: z.string(),
  subdomain: z.string(),
  domain: z.object({ uuid: z.string(), domain: z.string() }),
  allocation: serverAllocationSchema.nullable(),
  created: z.string(),
});

export const subdomainFormSchema = z.object({
  subdomain: z
    .string()
    .min(3)
    .max(32)
    .regex(/^[a-zA-Z0-9]+$/),
  domainUuid: z.uuid(),
  allocationUuid: z.uuid(),
});

export const subdomainUpdateFormSchema = z.object({
  subdomain: z.string(),
  domainUuid: z.string(),
  allocationUuid: z.uuid(),
});

export const adminSubdomainEntrySchema = z.object({
  uuid: z.string(),
  subdomain: z.string(),
  server: z.object({ uuid: z.string(), name: z.string() }).nullable(),
  allocation: serverAllocationSchema.nullable(),
  created: z.string(),
});
