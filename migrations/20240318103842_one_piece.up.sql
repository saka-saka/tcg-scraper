-- Add up migration script here
create type op_type_enum as enum('Leader', 'Event', 'Character', 'Stage');
create type op_rarity_enum as enum('SP', 'R', 'SEC', 'C', 'P', 'UC', 'SR', 'L');
create table one_piece(
	code text PRIMARY KEY,
	name text not null,
	img_src text not null,
	rarity text not null,
	type op_type_enum not null,
	get_info text not  null,
	set_name text not null
);
