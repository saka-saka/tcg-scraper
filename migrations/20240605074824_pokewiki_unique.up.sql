-- Add up migration script here
ALTER TABLE pokewiki ADD CONSTRAINT number_name_rarity_exp_code_key UNIQUE(number, name, rarity, exp_code);
