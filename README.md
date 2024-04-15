# README

## Introduction

This project is an authorization_server for NATS, working with a custom postgreSQL database

## Endpoints

- GET /jwt/v1/accounts/ --> Return 200, used by NATS to test if the endpoint is alive
- GET /jwt/v1/accounts/:account_id --> Return 200 with the account jwt or 404

## Pre-requisites

### Environment variables

- AUTHORIZATION_DB_CONNECTION_STRING, mandatory
- AUTHORIZATION_HOST, optional. Default: 127.0.0.1
- AUTHORIZATION_PORT, optional. Default: 9091

### PostgreSQL Setup
A PostgreSQL Database, with a table named **nats**. The service have been tested using Supabase

Fields currently used:
- id: uuid
- nsc_account_id: varchar
- creds_admin: text (optional, edit the test if there is none)
- creds_user: text (optional, edit the test if there is none)
- account_jwt: text