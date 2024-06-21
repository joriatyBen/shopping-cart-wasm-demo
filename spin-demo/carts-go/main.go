package main

import (
	"net/http"

	spinhttp "github.com/fermyon/spin/sdk/go/v2/http"
	"github.com/julienschmidt/httprouter"
)

func Dispatch(w http.ResponseWriter, req *http.Request) {
	router := httprouter.New()

	router.GET("/carts-go/:cartId", HandleGetCarts)
	router.GET("/carts-go/:cartId/items", HandleGetCartsItems)
	router.POST("/carts-go/:cartId/items", HandlePostCartsItems)
	router.PATCH("/carts-go/:cartId/items", HandlePatchCartsItems)
	router.DELETE("/carts-go/:cartId/items", HandleDeleteCartsItems)
	router.DELETE("/carts-go/:cartId/item/:itemId", HandleDeleteCartsItem)

	router.ServeHTTP(w, req)
}

func init() {
	spinhttp.Handle(Dispatch)
}

func main() {}
