-- Your SQL goes here
CREATE TABLE scores (
  id SERIAL PRIMARY KEY,
  player VARCHAR NOT NULL,
  n_turn INTEGER NOT NULL,
  median_time INTEGER NOT NULL,
  disks INTEGER NOT NULL,
  creation_date TIMESTAMP NOT NULL DEFAULT now()
);
