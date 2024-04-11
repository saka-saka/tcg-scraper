-- Add down migration script here
ALTER TABLE one_piece ALTER COLUMN rarity TYPE TEXT COLLATE 'SP' USING 'SP';
