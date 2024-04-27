-- Add migration script here
CREATE TABLE if not exists versions (
                version integer not null PRIMARY KEY, 
                version_date text not null,
                device text not null,
                selected_distance text not null);