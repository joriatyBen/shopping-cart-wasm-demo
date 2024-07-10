.PHONY: build push run

version?=latest

build:
	 mvn package

push:
	docker build -t ghcr.io/joriatyben/carts-java:${version} . && docker push ghcr.io/joriatyben/carts-java:${version}

run:
	docker run  --network=host --rm -p 8081:80 -e PG_HOST=localhost -e PG_PORT=5432 ghcr.io/joriatyben/carts-java:${version}
