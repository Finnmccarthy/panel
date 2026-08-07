DROP INDEX "user_security_keys_user_uuid_credential_id_idx";
DELETE FROM "user_security_keys" WHERE "uuid" IN (SELECT "uuid" FROM (SELECT "uuid", row_number() OVER (PARTITION BY "credential_id" ORDER BY "created", "uuid") AS "rn" FROM "user_security_keys") "ranked" WHERE "rn" > 1);
CREATE UNIQUE INDEX "user_security_keys_credential_id_idx" ON "user_security_keys" ("credential_id");
