docker compose down -v
docker compose up -d
# wait a few seconds for Postgres
export DATABASE_URL="postgres://postgres:postgres@localhost:5433/postgres"
sqlx migrate run