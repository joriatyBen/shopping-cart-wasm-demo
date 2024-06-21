package main

import (
	"fmt"
	"net/http"

	"github.com/julienschmidt/httprouter"
)

func HandleGetCarts(w http.ResponseWriter, _ *http.Request, params httprouter.Params) {
	fmt.Fprintf(w, "GET /carts/%s", params.ByName("cartId"))
}

func HandleGetCartsItems(w http.ResponseWriter, _ *http.Request, params httprouter.Params) {
	fmt.Fprintf(w, "GET /carts/items/%s", params.ByName("cartId"))
}

func HandlePostCartsItems(w http.ResponseWriter, _ *http.Request, params httprouter.Params) {
	fmt.Fprintf(w, "POST /carts/items/%s", params.ByName("cartId"))
}

func HandlePatchCartsItems(w http.ResponseWriter, _ *http.Request, params httprouter.Params) {
	fmt.Fprintf(w, "PATCH /carts/items/%s", params.ByName("cartId"))
}

func HandleDeleteCartsItems(w http.ResponseWriter, _ *http.Request, params httprouter.Params) {
	fmt.Fprintf(w, "DELETE /carts/items/%s", params.ByName("cartId"))
}

func HandleDeleteCartsItem(w http.ResponseWriter, _ *http.Request, params httprouter.Params) {
	fmt.Fprintf(w, "DELETE /carts/items/%s/item/%s", params.ByName("cartId"), params.ByName("itemId"))
}
