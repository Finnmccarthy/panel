CREATE TABLE dev_0x7d8_minecraftversionchanger_installations (
	server_uuid UUID REFERENCES servers (uuid) ON DELETE SET NULL,
	type_ VARCHAR(255) NOT NULL,
	version_id VARCHAR(255),
	project_version_id VARCHAR(255),
	build VARCHAR(255) NOT NULL,
	installed TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX dev_0x7d8_minecraftversionchanger_installations_server_uuid_idx ON dev_0x7d8_minecraftversionchanger_installations (server_uuid);
CREATE INDEX dev_0x7d8_minecraftversionchanger_installations_type_idx ON dev_0x7d8_minecraftversionchanger_installations (type_);
CREATE INDEX dev_0x7d8_minecraftversionchanger_installations_version_id_idx ON dev_0x7d8_minecraftversionchanger_installations (version_id);
CREATE INDEX dev_0x7d8_minecraftversionchanger_installations_project_version_id_idx ON dev_0x7d8_minecraftversionchanger_installations (project_version_id);
