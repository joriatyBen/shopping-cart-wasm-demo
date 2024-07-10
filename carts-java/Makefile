.PHONY: build push run

build:
	 mvn package

push:
	docker build -t ghcr.io/joriatyben/carts-java:${version} . && docker push ghcr.io/joriatyben/carts-java:${version}

run:
	docker run carts-java --rm -d -p 8081:8081
