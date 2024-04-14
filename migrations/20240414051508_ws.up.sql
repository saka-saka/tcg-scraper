-- Add up migration script here

CREATE TABLE ws_progress(
	id SERIAL PRIMARY KEY,
	created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	current_page INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE ws_cards(
	code TEXT PRIMARY KEY,
	name TEXT NOT NULL,
	set_code TEXT NOT NULL,
	img_src TEXT NOT NULL,
	rarity TEXT,
	set_name TEXT NOT NULL
);

INSERT INTO ws_progress(id) VALUES(DEFAULT);
