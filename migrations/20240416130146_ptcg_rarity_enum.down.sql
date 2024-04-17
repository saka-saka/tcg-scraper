-- Add down migration script here
ALTER TABLE tcg_collector DROP COLUMN IF EXISTS rarity;
DROP TYPE IF EXISTS ptcg_rarity_enum;
