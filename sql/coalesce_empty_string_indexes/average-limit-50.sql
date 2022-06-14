SELECT
	"users"."id",
	"users"."email",
	"users"."name",
	"users"."initials",
	"users"."role",
	"users"."invitation_status",
	"users"."allow_authentication_token",
	"users"."authentication_token",
	"users"."current_sign_in_at",
	"users"."last_sign_in_at",
	"users"."created_at",
	"users"."deleted_at",
	"users"."enable_export_data",
	"users"."tenant_id",
	"users"."settings"
FROM
	"users"
WHERE
	(
		users.tenant_id = 7
	)
	AND (
		COALESCE(users.invitation_status, '') > 'Pending'
		OR COALESCE(users.invitation_status, '') < 'Pending'
	)
ORDER BY
	"users"."initials" ASC,
	"users"."id" ASC
LIMIT 50;