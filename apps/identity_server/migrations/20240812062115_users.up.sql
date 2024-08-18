CREATE TABLE "users"
(
	user_id BLOB PRIMARY KEY NOT NULL,
	pubkeys_jwks TEXT             NOT NULL
) STRICT;
