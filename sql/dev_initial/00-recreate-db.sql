-- name: stop-db
SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE usename = 'app_user' OR datname = 'app_db';
-- name: drop-app-db
DROP DATABASE IF EXISTS app_db;
-- name: drop-app-user
DROP USER IF EXISTS app_user;

-- name: create-app-user
CREATE USER app_user PASSWORD 'dev_only_pwd';
-- name: create-app-db
CREATE DATABASE app_db owner app_user ENCODING = 'UTF-8';