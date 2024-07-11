# DEPRECATED: cart
A microservices-demo service that provides shopping carts for users.

The original YAGNI shopping cart by Weave Works could be found [here](https://github.com/microservices-demo/carts)

# API Spec

Checkout the API Spec [here](https://github.com/joriatyBen/shopping-cart-wasm-demo/blob/main/carts-java/api-spec/carts.json)

# Build

## Java

`mvn package`

## Docker

`make build $$ make push version=<semantic-version>`

# Run

`mvn spring-boot:run`
local container: `make run`

# Endpoints

GET /carts-rs/
```shell
curl -X GET http://127.0.0.1:8081/carts-java/1
```

GET /carts-rs/items
```shell 
curl -X GET http://127.0.0.1:8081/carts-java/1/items
```

POST /carts-rs/items
```shell
curl -X POST http://127.0.0.1:8081/carts-java/1/items \
-H "Content-Type: application/json" \
-d '{
"itemId": 123,
"quantity": 2,
"price": 19.99
}'
```

PATCH /carts-rs/items
```shell
curl -X PATCH http://127.0.0.1:8081/carts-java/1/items \
-H "Content-Type: application/json" \
-d '{
"itemId": 123,
"quantity": 3,
"price": 17.99
}'
```

DELETE /carts-rs/items
```shell
curl -X DELETE http://127.0.0.1:8081/carts-java/1/items
```

DELETE /carts-rs/items
```shell
curl -X DELETE http://127.0.0.1:8081/carts-java/1/items/123
```