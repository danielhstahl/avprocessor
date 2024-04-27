-- Add migration script here
CREATE TABLE if not exists speakers_for_camilla (
                version text not null, 
                speaker text not null, 
                crossover integer, 
                delay real not null, 
                gain real not null,
                is_subwoofer integer not null, 
                PRIMARY KEY (version, speaker));