package main

import (
	"fmt"
	"net/http"
	"time"

	"github.com/julienschmidt/httprouter"
)

type Handler struct {
	config Config
}

func (h Handler) ServeHTTP(w http.ResponseWriter, req *http.Request) {
	router := httprouter.New()
	controller := Controller{Cfg: h.config, Start: time.Now().UnixMilli()}

	router.GET("/carts-go/:cartId", controller.HandleGetCarts)
	router.GET("/carts-go/:cartId/items", controller.HandleGetCartsItems)
	router.POST("/carts-go/:cartId/items", controller.HandlePostCartsItems)
	router.PATCH("/carts-go/:cartId/items", controller.HandlePatchCartsItems)
	router.DELETE("/carts-go/:cartId/items", controller.HandleDeleteCartsItems)
	router.DELETE("/carts-go/:cartId/item/:itemId", controller.HandleDeleteCartsItem)

	router.ServeHTTP(w, req)
}

func main() {
	config := ParseConfig()

	server := http.Server{
		Addr:    config.Listen,
		Handler: Handler{config},
	}

	fmt.Printf("listening on %s...", config.Listen)

	err := server.ListenAndServe()
	if err != nil {
		panic(err)
	}
}
