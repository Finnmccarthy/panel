CREATE TYPE dev_0x7d8_subdomainmanager_provider AS ENUM ('CLOUDFLARE');

CREATE TABLE dev_0x7d8_subdomainmanager_domains (
    uuid uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
    provider dev_0x7d8_subdomainmanager_provider NOT NULL DEFAULT 'CLOUDFLARE',
    eggs UUID[] NOT NULL DEFAULT '{}',
    domain VARCHAR(255) NOT NULL,
    disallowed_subdomains_regexes TEXT[] NOT NULL DEFAULT '{}',
    provider_config JSONB NOT NULL DEFAULT '{}'
);

CREATE TABLE dev_0x7d8_subdomainmanager_server_subdomains (
    uuid uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
    subdomain VARCHAR(255) NOT NULL,
    server_uuid UUID REFERENCES servers(uuid) ON DELETE SET NULL,
    domain_uuid UUID NOT NULL REFERENCES dev_0x7d8_subdomainmanager_domains(uuid) ON DELETE CASCADE,
    allocation_uuid UUID REFERENCES server_allocations(uuid) ON DELETE SET NULL,
    created TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(subdomain, domain_uuid)
);

ALTER TABLE servers ADD COLUMN subdomain_limit INTEGER NOT NULL DEFAULT 0;
