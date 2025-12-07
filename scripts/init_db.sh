#!/bin/bash
set -e

# check if sqlx-cli is installed
if ! command -v sqlx &> /dev/null; then
    echo "sqlx-cli is not installed. Installing..."
    cargo install sqlx-cli --no-default-features --features rustls,postgres
fi

# check if docker is running
if ! docker info > /dev/null 2>&1; then
    echo "Docker is not running. Please start Docker."
    exit 1
fi

DB_USER=${POSTGRES_USER:=postgres}
DB_PASSWORD=${POSTGRES_PASSWORD:=password}
DB_NAME=${POSTGRES_DB:=drizzle}
DB_PORT=${POSTGRES_PORT:=5442}

# Launch postgres
if [[ -z "${SKIP_DOCKER}" ]]; then
  # check if container exists and remove it if it does
  if [ "$(docker ps -aq -f name=drizzle-postgres)" ]; then
    docker rm -f drizzle-postgres
  fi

  docker run \
      --name drizzle-postgres \
      -e POSTGRES_USER=${DB_USER} \
      -e POSTGRES_PASSWORD=${DB_PASSWORD} \
      -e POSTGRES_DB=${DB_NAME} \
      -p "${DB_PORT}":5432 \
      -d postgres \
      postgres -N 1000
      # ^ Increased max connections for testing
fi

# Keep pining Postgres until it's ready to accept commands
export PGPASSWORD="${DB_PASSWORD}"
until docker exec drizzle-postgres psql -U "${DB_USER}" -d "${DB_NAME}" -c '\q'; do
  >&2 echo "Postgres is still unavailable - sleeping"
  sleep 1
done

>&2 echo "Postgres is up and running on port ${DB_PORT}!"

export DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}
echo "DATABASE_URL=$DATABASE_URL" > .env

# Create database and run migrations
sqlx database create
sqlx migrate run --source control-plane/crates/storage/migrations

echo "✅ Database initialized and migrations run!"
