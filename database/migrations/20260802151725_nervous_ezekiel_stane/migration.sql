CREATE TABLE "node_database_agent_hosts" (
	"node_uuid" uuid,
	"database_agent_host_uuid" uuid,
	"created" timestamp DEFAULT now() NOT NULL,
	CONSTRAINT "node_database_agent_hosts_pk" PRIMARY KEY("node_uuid","database_agent_host_uuid")
);

CREATE TABLE "node_database_hosts" (
	"node_uuid" uuid,
	"database_host_uuid" uuid,
	"created" timestamp DEFAULT now() NOT NULL,
	CONSTRAINT "node_database_hosts_pk" PRIMARY KEY("node_uuid","database_host_uuid")
);

CREATE INDEX "node_database_agent_hosts_node_uuid_idx" ON "node_database_agent_hosts" ("node_uuid");
CREATE INDEX "node_database_agent_hosts_database_agent_host_uuid_idx" ON "node_database_agent_hosts" ("database_agent_host_uuid");
CREATE INDEX "node_database_hosts_node_uuid_idx" ON "node_database_hosts" ("node_uuid");
CREATE INDEX "node_database_hosts_database_host_uuid_idx" ON "node_database_hosts" ("database_host_uuid");
ALTER TABLE "node_database_agent_hosts" ADD CONSTRAINT "node_database_agent_hosts_node_uuid_nodes_uuid_fkey" FOREIGN KEY ("node_uuid") REFERENCES "nodes"("uuid") ON DELETE CASCADE;
ALTER TABLE "node_database_agent_hosts" ADD CONSTRAINT "node_database_agent_hosts_h5C1pY9CSwTJ_fkey" FOREIGN KEY ("database_agent_host_uuid") REFERENCES "database_agent_hosts"("uuid") ON DELETE CASCADE;
ALTER TABLE "node_database_hosts" ADD CONSTRAINT "node_database_hosts_node_uuid_nodes_uuid_fkey" FOREIGN KEY ("node_uuid") REFERENCES "nodes"("uuid") ON DELETE CASCADE;
ALTER TABLE "node_database_hosts" ADD CONSTRAINT "node_database_hosts_database_host_uuid_database_hosts_uuid_fkey" FOREIGN KEY ("database_host_uuid") REFERENCES "database_hosts"("uuid") ON DELETE CASCADE;