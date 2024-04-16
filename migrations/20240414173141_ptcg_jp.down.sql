-- Add down migration script here
DROP TABLE IF EXISTS ptcg_jp_expansions;
DROP TABLE IF EXISTS tcg_collector;

ALTER TABLE pokemon_trainer_printing DROP COLUMN IF EXISTS name_en;
ALTER TABLE pokemon_trainer_printing DROP COLUMN IF EXISTS skill1_name_en;
ALTER TABLE pokemon_trainer_printing DROP COLUMN IF EXISTS skill1_damage;
ALTER TABLE pokemon_trainer_printing DROP COLUMN IF EXISTS card_description_en;
ALTER TABLE pokemon_trainer_printing DROP CONSTRAINT expansion_code_number_key;
