
`.open db/settings.sqlite`

## Table definitions

Will need to make sure filters are translatable from REW.
`CREATE TABLE filters (version text, index integer, speaker text, freq real, gain real, q real, PRIMARY KEY (version, index, speaker));`

`CREATE TABLE speakers (version text, speaker text, crossover integer, delay integer, gain integer, is_subwoofer boolean PRIMARY KEY (version, speaker));`

Description should include something about the version.
`CREATE TABLE versions (version text PRIMARY KEY);`