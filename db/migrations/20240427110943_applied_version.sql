-- Add migration script here
CREATE TABLE if not exists applied_version (
                version integer not null PRIMARY KEY);