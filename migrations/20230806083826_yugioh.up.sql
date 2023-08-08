-- Add up migration script here
create table yugioh_expansion_link(
    url text not null unique
);

create table yugioh_printing_link(
    url text not null unique
);

create table yugioh_printing_detail(
    card_id text not null,
    name_jp text not null,
    name_en text not null,
    rarity text not null,
    number text not null,
    release_date text not null,
    remark text,
    expansion_name text not null,
    expansion_code text not null,
    UNIQUE(card_id, expansion_name, rarity)
);
