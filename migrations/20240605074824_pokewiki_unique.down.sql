-- Add down migration script here
ALTER TABLE pokewiki DROP CONSTRAINT IF EXISTS number_name_rarity_exp_code_key;
