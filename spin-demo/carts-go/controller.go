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
	"time"

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
	now := time.Now().UnixMilli()

	cartId, err := strconv.Atoi(params.ByName("cartId"))
	if err != nil {
		responseBadRequest(now, w, err)
		return
	}

	rows, err := connectDb().Query("SELECT item_id, quantity, price FROM cart.cart_items WHERE cart_id = $1", int32(cartId))
	assert(err)
	defer rows.Close()

	if rows.Next() {
		responseJson(now, w, GetCartsResponse{uint(cartId)})
	} else {
		responseNotFound(now, w)
	}
}

func HandleGetCartsItems(w http.ResponseWriter, _ *http.Request, params httprouter.Params) {
	now := time.Now().UnixMilli()

	cartId, err := strconv.Atoi(params.ByName("cartId"))
	if err != nil {
		responseBadRequest(now, w, err)
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
		responseNotFound(now, w)
	} else {
		responseJson(now, w, items)
	}
}

func HandlePostCartsItems(w http.ResponseWriter, req *http.Request, params httprouter.Params) {
	now := time.Now().UnixMilli()

	cartId, err := strconv.Atoi(params.ByName("cartId"))
	if err != nil {
		responseBadRequest(now, w, err)
		return
	}

	body, err := io.ReadAll(req.Body)
	if err != nil {
		responseBadRequest(now, w, err)
		return
	}

	item := CartItem{}
	if err = json.Unmarshal(body, &item); err != nil {
		responseBadRequest(now, w, err)
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
		responseBadRequest(now, w, errors.New("duplicate id"))
	} else {
		responseJson(now, w, item)
	}
}

func HandlePatchCartsItems(w http.ResponseWriter, req *http.Request, params httprouter.Params) {
	now := time.Now().UnixMilli()

	cartId, err := strconv.Atoi(params.ByName("cartId"))
	if err != nil {
		responseBadRequest(now, w, err)
		return
	}

	body, err := io.ReadAll(req.Body)
	if err != nil {
		responseBadRequest(now, w, err)
		return
	}

	patch := CartItemPatch{}
	if err = json.Unmarshal(body, &patch); err != nil {
		responseBadRequest(now, w, err)
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

		responseJson(now, w, item)
	} else {
		responseNotFound(now, w)
	}
}

func HandleDeleteCartsItems(w http.ResponseWriter, _ *http.Request, params httprouter.Params) {
	now := time.Now().UnixMilli()

	cartId, err := strconv.Atoi(params.ByName("cartId"))
	if err != nil {
		responseBadRequest(now, w, err)
		return
	}

	result, err := connectDb().Exec("DELETE FROM cart.cart_items WHERE cart_id = $1", int32(cartId))
	assert(err)

	rowsAffected, err := result.RowsAffected()
	assert(err)

	if rowsAffected == 0 {
		responseNotFound(now, w)
	} else {
		responseEmpty(now, w)
	}
}

func HandleDeleteCartsItem(w http.ResponseWriter, _ *http.Request, params httprouter.Params) {
	now := time.Now().UnixMilli()

	cartId, err := strconv.Atoi(params.ByName("cartId"))
	if err != nil {
		responseBadRequest(now, w, err)
		return
	}

	itemId, err := strconv.Atoi(params.ByName("itemId"))
	if err != nil {
		responseBadRequest(now, w, err)
		return
	}

	result, err := connectDb().Exec("DELETE FROM cart.cart_items WHERE cart_id = $1 AND item_id = $2", int32(cartId), int32(itemId))
	assert(err)

	rowsAffected, err := result.RowsAffected()
	assert(err)

	if rowsAffected == 0 {
		responseNotFound(now, w)
	} else {
		responseEmpty(now, w)
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

func addProcessingTimeHeader(start int64, w http.ResponseWriter) {
	w.Header().Add("X-Processing-Time-Milliseconds", fmt.Sprintf("%v", time.Now().UnixMilli()-start))
}

func responseNotFound(start int64, w http.ResponseWriter) {
	addProcessingTimeHeader(start, w)

	w.WriteHeader(http.StatusNotFound)
	w.Write([]byte("not found"))
}

func responseBadRequest(start int64, w http.ResponseWriter, err error) {
	addProcessingTimeHeader(start, w)

	w.WriteHeader(http.StatusBadRequest)
	w.Write([]byte(err.Error()))
}

func responseJson(start int64, w http.ResponseWriter, data any) {
	addProcessingTimeHeader(start, w)

	json, err := json.Marshal(data)
	assert(err)

	w.Header().Add("Content-Type", "application/json")
	w.WriteHeader(http.StatusOK)
	w.Write(json)
}

func responseEmpty(start int64, w http.ResponseWriter) {
	addProcessingTimeHeader(start, w)

	w.WriteHeader(http.StatusOK)
}
