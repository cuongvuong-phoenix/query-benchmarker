UPDATE users
SET invitation_status = NULL
WHERE invitation_status = 'None';

CREATE INDEX index_users_on_tenant_id_and_non_null_invitation_status ON users (tenant_id, COALESCE(invitation_status, ''));