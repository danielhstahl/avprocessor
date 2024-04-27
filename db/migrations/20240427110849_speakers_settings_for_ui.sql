-- Add migration script here
CREATE TABLE if not exists speakers_settings_for_ui (
            version integer not null,
            speaker text not null, 
            crossover integer, 
            distance real not null, 
            gain real not null, 
            is_subwoofer integer not null, 
            PRIMARY KEY (version, speaker));