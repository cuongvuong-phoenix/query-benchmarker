-- Add down migration script here
CREATE INDEX index_users_on_tenant_id ON users (tenant_id);