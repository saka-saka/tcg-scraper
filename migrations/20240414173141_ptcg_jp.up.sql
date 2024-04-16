-- Add up migration script here
CREATE TABLE ptcg_jp_expansions(
	code TEXT PRIMARY KEY,
	name_en TEXT NOT NULL,
	exp_link TEXT,
	symbol_src TEXT,
	logo_src TEXT,
	release_date DATE NOT NULL
);

CREATE TABLE tcg_collector(
	name TEXT NOT NULL,
	number TEXT NOT NULL,
	exp_code TEXT NOT NULL,
	html TEXT NOT NULL,
	url TEXT NOT NULL
);

ALTER TABLE pokemon_trainer_expansion ALTER COLUMN release_date TYPE date USING NOW();
ALTER TABLE pokemon_trainer_printing ADD COLUMN name_en TEXT;
ALTER TABLE pokemon_trainer_printing ADD COLUMN skill1_name_en TEXT;
ALTER TABLE pokemon_trainer_printing ADD COLUMN skill1_damage TEXT;
ALTER TABLE pokemon_trainer_printing ADD COLUMN card_description_en TEXT;
ALTER TABLE pokemon_trainer_printing ADD CONSTRAINT expansion_code_number_key UNIQUE(name, number, expansion_code);
