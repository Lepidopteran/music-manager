-- Add down migration script here

ALTER TABLE songs DROP COLUMN `added_at`;
ALTER TABLE songs DROP COLUMN `updated_at`;
ALTER TABLE songs DROP COLUMN `file_created_at`;
