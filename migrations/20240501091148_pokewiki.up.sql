-- Add up migration script here
CREATE TABLE pokewiki(number TEXT NOT NULL, name TEXT NOT NULL, rarity ptcg_rarity_enum NOT NULL, exp_code TEXT NOT NULL);
