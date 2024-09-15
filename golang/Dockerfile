FROM golang:1.23.1-alpine AS build

COPY . /build
WORKDIR /build

RUN GOOS=linux GOARCH=amd64 CGO_ENABLED=0 go build -ldflags="-s -w" .

FROM scratch

COPY --from=build /build/shoppingcart /

CMD ["/shoppingcart"]