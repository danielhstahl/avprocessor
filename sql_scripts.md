
`.open db/settings.sqlite`

## Table definitions

Will need to make sure filters are translatable from REW.
`CREATE TABLE filters (version text, speaker text, type text, freq real, gain real, q real, key real, PRIMARY KEY (version, speaker, key));`

`CREATE TABLE speaker (version text, speaker text, crossover integer, delay integer, gain integer, is_subwoofer boolean PRIMARY KEY (version, speaker));`

//`CREATE TABLE delays (version text, speaker text, delay integer, PRIMARY KEY (version, speaker));`

Description should include something about the version.
`CREATE TABLE versions (version text PRIMARY KEY, description text);`