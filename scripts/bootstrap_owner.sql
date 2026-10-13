-- Run once via interactive psql against a migrated, trusted database.
-- psql -X -v ON_ERROR_STOP=1 -f scripts/bootstrap_owner.sql
\prompt 'Owner full name: ' owner_name
\prompt 'Owner email: ' owner_email
\prompt 'Owner initial password (input visible; use a private terminal): ' owner_password
INSERT INTO auth_users (name, email, password_hash, role)
SELECT :'owner_name', lower(:'owner_email'), crypt(:'owner_password', gen_salt('bf',12)), 'owner'
WHERE NOT EXISTS (SELECT 1 FROM auth_users WHERE role = 'owner');
\unset owner_password
