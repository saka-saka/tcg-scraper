-- Add up migration script here
ALTER TABLE one_piece ALTER COLUMN rarity TYPE op_rarity_enum USING 'SP';
