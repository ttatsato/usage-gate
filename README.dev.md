
# Ready env file
```
cp .env.example .env
```

#  Run Database

```
docker compose up -d
```

# Run API Server

```
cargo run
```


# TIPS

## Create migration file
```
sqlx migrate add <MIGRATION_NAME>
```

### Execute migration
```
sqlx migrate run
```

### Check Out If Created Table
```
docker exec -it usage-gate-db psql -U usage_gate -d usage_gate -c '\dt'
docker exec -it usage-gate-db psql -U usage_gate -d usage_gate -c '\d tenants'
```
