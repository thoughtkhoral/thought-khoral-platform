FROM docker.io/pgvector/pgvector:0.8.1-pg17-trixie

COPY thought-khoral-platform/postgres/0000-databases.sql /docker-entrypoint-initdb.d/0000-databases.sql
COPY thought-khoral-room-gateway/migrations/ /docker-entrypoint-initdb.d/
