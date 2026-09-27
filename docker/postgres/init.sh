#!/usr/bin/env bash
# Runs once, on an empty volume, as part of the official image's
# docker-entrypoint-initdb.d sequence.
set -euo pipefail

psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" <<-EOSQL
  CREATE ROLE life_pixel LOGIN PASSWORD '$LP_DB_PASSWORD';
  CREATE DATABASE life_pixel OWNER life_pixel;
  REVOKE CONNECT ON DATABASE life_pixel FROM PUBLIC;

  CREATE ROLE life_pixel_admin LOGIN PASSWORD '$LPA_DB_PASSWORD';
  CREATE DATABASE life_pixel_admin OWNER life_pixel_admin;
  REVOKE CONNECT ON DATABASE life_pixel_admin FROM PUBLIC;
EOSQL

# pg_trgm and citext are trusted extensions since Postgres 13: their owning
# role creates them with a plain CREATE EXTENSION, no superuser needed.
