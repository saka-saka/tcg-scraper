-- Add up migration script here
create table bigweb_pokemon_expansion(
    id uuid primary key,
    code text not null,
    is_sync boolean not null default false,
    item_count integer,
    updated_at timestamptz not null default NOW(),
    name text not null
);

create table bigweb_pokemon_printing(
    id uuid primary key,
    name text not null,
    number text,
    rarity text,
    sale_price integer,
    expansion_id uuid not null,
    remark text,
    image_downloaded boolean not null default false,
    last_fetched_at timestamptz,
    CONSTRAINT fk_bigweb_pokemon_expansion
        FOREIGN KEY(expansion_id)
            REFERENCES bigweb_pokemon_expansion(id)
);

create table pokemon_trainer_expansion(
    id uuid not null primary key,
    series text not null,
    release_date text not null,
    updated_at timestamptz not null default NOW(),
    code text not null unique,
    name text not null
);

create table pokemon_trainer_printing(
    code text not null unique,
    name text not null,
    kind text not null,
    number text not null,
    rarity text,
    expansion_code text not null
);

create table pokemon_trainer_fetchable_card(
    code text not null unique,
    fetched boolean not null default false,
    expansion_code text not null
)

