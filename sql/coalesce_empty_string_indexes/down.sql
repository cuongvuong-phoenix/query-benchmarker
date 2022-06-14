DROP INDEX index_users_on_tenant_id_and_non_null_invitation_status;

UPDATE users
SET invitation_status = 'None'
WHERE invitation_status IS NULL;