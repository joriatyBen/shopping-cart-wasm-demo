#### Deploy wasmcloud app
1. run `make wasmcloud` in ../../ansible
2. `k port-forward -n wasmcloud svc/nats 4222:4222` 
3. `wash config put default-postgres \
    POSTGRES_HOST=postgres-postgresql.postgres \
    POSTGRES_PORT=5432 \
    POSTGRES_USERNAME=myuser \
    POSTGRES_PASSWORD=p05tgr3$ \
    POSTGRES_DATABASE=products \
    POSTGRES_TLS_REQUIRED=false
    `
4. `wash app deploy --replace wadm.yaml`
5. port forward host on which app is running `k port-forward -n wasmcloud wasmcloud-host-xxxx-xxx 8080`
5. test: `curl 127.0.0.1:8080/carts-wcrs/10`