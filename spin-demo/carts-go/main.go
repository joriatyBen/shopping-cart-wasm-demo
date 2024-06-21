package main

import (
	"net/http"
	"time"

	spinhttp "github.com/fermyon/spin/sdk/go/v2/http"
	"github.com/julienschmidt/httprouter"
)

func Dispatch(w http.ResponseWriter, req *http.Request) {

	router := httprouter.New()
	controller := Controller{Start: time.Now().UnixMilli()}

	router.GET("/carts-go/:cartId", controller.HandleGetCarts)
	router.GET("/carts-go/:cartId/items", controller.HandleGetCartsItems)
	router.POST("/carts-go/:cartId/items", controller.HandlePostCartsItems)
	router.PATCH("/carts-go/:cartId/items", controller.HandlePatchCartsItems)
	router.DELETE("/carts-go/:cartId/items", controller.HandleDeleteCartsItems)
	router.DELETE("/carts-go/:cartId/item/:itemId", controller.HandleDeleteCartsItem)

	router.ServeHTTP(w, req)
}

func init() {
	spinhttp.Handle(Dispatch)
}

func main() {}
