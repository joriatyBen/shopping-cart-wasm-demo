package main

import (
	"database/sql"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/http"
	"os"
	"strconv"
	"strings"

	spinpg "github.com/fermyon/spin/sdk/go/v2/pg"
	"github.com/julienschmidt/httprouter"
)

type CartItem struct {
	Id       uint    `json:"itemId"`
	Quantity uint    `json:"quantity"`
	Price    float64 `json:"price"`
}

type CartItemPatch struct {
	Id       uint     `json:"itemId"`
	Quantity *uint    `json:"quantity"`
	Price    *float64 `json:"price"`
}

type GetCartsResponse struct {
	Id uint `json:"customerId"`
}

func HandleGetCarts(w http.ResponseWriter, _ *http.Request, params httprouter.Params) {
	cartId, err := strconv.Atoi(params.ByName("cartId"))
	if err != nil {
		responseBadRequest(w, err)
		return
	}

	rows, err := connectDb().Query("SELECT item_id, quantity, price FROM cart.cart_items WHERE cart_id = $1", int32(cartId))
	assert(err)
	defer rows.Close()

	if rows.Next() {
		responseJson(w, GetCartsResponse{uint(cartId)})
	} else {
		responseNotFound(w)
	}
}

func HandleGetCartsItems(w http.ResponseWriter, _ *http.Request, params httprouter.Params) {
	cartId, err := strconv.Atoi(params.ByName("cartId"))
	if err != nil {
		responseBadRequest(w, err)
		return
	}

	rows, err := connectDb().Query("SELECT item_id, quantity, price FROM cart.cart_items WHERE cart_id = $1", int32(cartId))
	assert(err)
	defer rows.Close()

	items := make([]CartItem, 0, 5)
	for rows.Next() {
		item := CartItem{}
		rows.Scan(&item.Id, &item.Quantity, &item.Price)

		items = append(items, item)
	}

	if len(items) == 0 {
		responseNotFound(w)
	} else {
		responseJson(w, items)
	}
}

func HandlePostCartsItems(w http.ResponseWriter, req *http.Request, params httprouter.Params) {
	cartId, err := strconv.Atoi(params.ByName("cartId"))
	if err != nil {
		responseBadRequest(w, err)
		return
	}

	body, err := io.ReadAll(req.Body)
	if err != nil {
		responseBadRequest(w, err)
		return
	}

	item := CartItem{}
	if err = json.Unmarshal(body, &item); err != nil {
		responseBadRequest(w, err)
		return
	}

	result, err := connectDb().Exec("INSERT INTO cart.cart_items VALUES($1, $2, $3, $4) ON CONFLICT DO NOTHING;",
		int32(cartId),
		int32(item.Id),
		int32(item.Quantity),
		item.Price,
	)
	assert(err)

	rowsAffected, err := result.RowsAffected()
	assert(err)

	if rowsAffected == 0 {
		responseBadRequest(w, errors.New("duplicate id"))
	} else {
		responseJson(w, item)
	}
}

func HandlePatchCartsItems(w http.ResponseWriter, req *http.Request, params httprouter.Params) {
	cartId, err := strconv.Atoi(params.ByName("cartId"))
	if err != nil {
		responseBadRequest(w, err)
		return
	}

	body, err := io.ReadAll(req.Body)
	if err != nil {
		responseBadRequest(w, err)
		return
	}

	patch := CartItemPatch{}
	if err = json.Unmarshal(body, &patch); err != nil {
		responseBadRequest(w, err)
		return
	}

	mutations := make([]string, 0, 2)
	parameters := []any{int32(cartId), int32(patch.Id)}

	if patch.Quantity != nil {
		parameters = append(parameters, int32(*patch.Quantity))
		mutations = append(mutations, fmt.Sprintf("quantity = $%v", len(parameters)))
	}

	if patch.Price != nil {
		parameters = append(parameters, *patch.Price)
		mutations = append(mutations, fmt.Sprintf("price = $%v", len(parameters)))
	}

	connection := connectDb()
	_, err = connection.Exec(fmt.Sprintf(
		"UPDATE cart.cart_items SET %s WHERE cart_id = $1 AND item_id = $2", strings.Join(mutations, ", ")),
		parameters...,
	)
	assert(err)

	rows, err := connection.Query("SELECT item_id, quantity, price FROM cart.cart_items WHERE cart_id = $1 AND item_id = $2",
		int32(cartId),
		int32(patch.Id),
	)
	assert(err)
	defer rows.Close()

	if rows.Next() {
		item := CartItem{}
		rows.Scan(&item.Id, &item.Quantity, &item.Price)

		responseJson(w, item)
	} else {
		responseNotFound(w)
	}
}

func HandleDeleteCartsItems(w http.ResponseWriter, _ *http.Request, params httprouter.Params) {
	cartId, err := strconv.Atoi(params.ByName("cartId"))
	if err != nil {
		responseBadRequest(w, err)
		return
	}

	result, err := connectDb().Exec("DELETE FROM cart.cart_items WHERE cart_id = $1", int32(cartId))
	assert(err)

	rowsAffected, err := result.RowsAffected()
	assert(err)

	if rowsAffected == 0 {
		responseNotFound(w)
	} else {
		responseEmpty(w)
	}
}

func HandleDeleteCartsItem(w http.ResponseWriter, _ *http.Request, params httprouter.Params) {
	cartId, err := strconv.Atoi(params.ByName("cartId"))
	if err != nil {
		responseBadRequest(w, err)
		return
	}

	itemId, err := strconv.Atoi(params.ByName("itemId"))
	if err != nil {
		responseBadRequest(w, err)
		return
	}

	result, err := connectDb().Exec("DELETE FROM cart.cart_items WHERE cart_id = $1 AND item_id = $2", int32(cartId), int32(itemId))
	assert(err)

	rowsAffected, err := result.RowsAffected()
	assert(err)

	if rowsAffected == 0 {
		responseNotFound(w)
	} else {
		responseEmpty(w)
	}
}

func connectDb() *sql.DB {
	return spinpg.Open(os.Getenv("DB_URL"))

}

func assert(e error) {
	if e != nil {
		panic(e)
	}
}

func responseNotFound(w http.ResponseWriter) {
	w.WriteHeader(http.StatusNotFound)
	w.Write([]byte("not found"))
}

func responseBadRequest(w http.ResponseWriter, err error) {
	w.WriteHeader(http.StatusBadRequest)
	w.Write([]byte(err.Error()))
}

func responseJson(w http.ResponseWriter, data any) {
	json, err := json.Marshal(data)
	assert(err)

	w.Header().Add("Content-Type", "application/json")
	w.WriteHeader(http.StatusOK)
	w.Write(json)
}

func responseEmpty(w http.ResponseWriter) {
	w.WriteHeader(http.StatusOK)
}
