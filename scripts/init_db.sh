#!/usr/bin/env bash
set -eox pipefail

echo $PATH

if ! [ -x "$(command -v psql)" ]
then
  echo "Error: psql is not installed."
  exit 1
fi

if ! [ -x "$(command -v cargo sqlx)" ]
then
  echo "Error: sqlx is not installed."
  echo "Use:"
  echo "    cargo install sqlx-cli --no-default-features --features postgres,rustls"
  echo "to install it."
  exit 1
fi

# Default config
DB_USER=${POSTGRES_USER:=postgres}
DB_PASSWORD=${POSTGRES_PASSWORD:=postgres}
DB_NAME=${POSTGRES_DB:=newsletter}
DB_PORT=${POSTGRES_PORT:=5432}

# Launch PostgreSQL using Docker
if [[ -z "${SKIP_DOCKER}" ]]
then
  docker run \
    -e POSTGRES_USER=$DB_USER \
    -e POSTGRES_PASSWORD=$DB_PASSWORD \
    -e POSTGRES_DB=$DB_NAME \
    -p $DB_PORT:5432 \
    -d postgres \
    postgres -N 1000
fi

# Create password variable for psql
export PGPASSWORD=$DB_PASSWORD

# Wait for PostgreSQL to be ready
until psql -h localhost -U $DB_USER -d "postgres" -c '\q'
do
  echo "Postgres is unavailable - sleeping"
  sleep 1
done

>&2 echo "Postgres is up and running on port ${DB_PORT} - running migrations now!"

export DATABASE_URL="postgresql://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}"

sqlx database create
sqlx migrate run

echo "Postgres has been migrated, ready to go!"