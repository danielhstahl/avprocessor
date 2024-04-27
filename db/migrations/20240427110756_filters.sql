-- Add migration script here
CREATE TABLE if not exists filters (
    version integer not null, 
    filter_index integer not null, 
    speaker text not null, 
    freq integer not null, 
    gain real not null, 
    q real not null, 
    PRIMARY KEY (version, filter_index, speaker)
);