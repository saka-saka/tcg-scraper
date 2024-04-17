-- Add up migration script here
CREATE TYPE ptcg_rarity_enum AS ENUM(
    'UR',
    'SSR',
    'ACE',
    'HR',
    'SR',
    'SAR',
    'CSR',
    'AR',
    'CHR',
    'S',
    'A',
    'H',
    'K',
    'PR',
    'RRR',
    'RR',
    'R',
    'U',
    'C',
    'TR',
    'TD',
    'Unknown'
);

ALTER TABLE tcg_collector ADD COLUMN rarity ptcg_rarity_enum;
