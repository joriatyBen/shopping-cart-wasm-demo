package main

import (
	"database/sql"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/http"
	"strconv"
	"strings"
	"time"

	"github.com/fermyon/spin/sdk/go/v2/pg"
	"github.com/fermyon/spin/sdk/go/v2/variables"
	"github.com/julienschmidt/httprouter"
)

type Controller struct {
	Start int64

	accumulatedTimeDb int64
	startDb           int64

	cartId int
	itemId int
	body   []byte

	err error
}

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

type connectionWrapper struct {
	db         *sql.DB
	controller *Controller
}

func (c *connectionWrapper) Query(query string, args ...any) (*sql.Rows, error) {
	c.controller.startDbTimer()
	rows, err := c.db.Query(query, args...)
	c.controller.stopDbTimer()

	return rows, err
}

func (c *connectionWrapper) Exec(query string, args ...any) (sql.Result, error) {
	c.controller.startDbTimer()
	result, err := c.db.Exec(query, args...)
	c.controller.stopDbTimer()

	return result, err
}

func (c *Controller) startDbTimer() {
	c.startDb = time.Now().UnixMilli()
}

func (c *Controller) stopDbTimer() {
	c.accumulatedTimeDb = c.accumulatedTimeDb + (time.Now().UnixMilli() - c.startDb)
}

func (c *Controller) HandleGetCarts(w http.ResponseWriter, _ *http.Request, params httprouter.Params) {
	c.fetchCartId(params)
	if c.err != nil {
		c.responseBadRequest(w, c.err)
		return
	}

	rows, err := c.connectDb().Query("SELECT item_id, quantity, price FROM cart.cart_items WHERE cart_id = $1", int32(c.cartId))
	assert(err)
	defer rows.Close()

	if rows.Next() {
		c.responseJson(w, GetCartsResponse{uint(c.cartId)})
	} else {
		c.responseNotFound(w)
	}
}

func (c *Controller) HandleGetCartsItems(w http.ResponseWriter, _ *http.Request, params httprouter.Params) {
	c.fetchCartId(params)
	if c.err != nil {
		c.responseBadRequest(w, c.err)
		return
	}

	rows, err := c.connectDb().Query("SELECT item_id, quantity, price FROM cart.cart_items WHERE cart_id = $1", int32(c.cartId))
	assert(err)
	defer rows.Close()

	items := make([]CartItem, 0, 5)
	for rows.Next() {
		item := CartItem{}
		rows.Scan(&item.Id, &item.Quantity, &item.Price)

		items = append(items, item)
	}

	if len(items) == 0 {
		c.responseNotFound(w)
	} else {
		c.responseJson(w, items)
	}
}

func (c *Controller) HandlePostCartsItems(w http.ResponseWriter, req *http.Request, params httprouter.Params) {
	c.fetchCartId(params)
	c.fetchBody(req)
	if c.err != nil {
		c.responseBadRequest(w, c.err)
		return
	}

	item := CartItem{}
	if err := json.Unmarshal(c.body, &item); err != nil {
		c.responseBadRequest(w, err)
		return
	}

	result, err := c.connectDb().Exec("INSERT INTO cart.cart_items VALUES($1, $2, $3, $4) ON CONFLICT DO NOTHING;",
		int32(c.cartId),
		int32(item.Id),
		int32(item.Quantity),
		item.Price,
	)
	assert(err)

	rowsAffected, err := result.RowsAffected()
	assert(err)

	if rowsAffected == 0 {
		c.responseBadRequest(w, errors.New("duplicate id"))
	} else {
		c.responseJson(w, item)
	}
}

func (c *Controller) HandlePatchCartsItems(w http.ResponseWriter, req *http.Request, params httprouter.Params) {
	c.fetchCartId(params)
	c.fetchBody(req)
	if c.err != nil {
		c.responseBadRequest(w, c.err)
		return
	}

	patch := CartItemPatch{}
	if err := json.Unmarshal(c.body, &patch); err != nil {
		c.responseBadRequest(w, err)
		return
	}

	mutations := make([]string, 0, 2)
	parameters := []any{int32(c.cartId), int32(patch.Id)}

	if patch.Quantity != nil {
		parameters = append(parameters, int32(*patch.Quantity))
		mutations = append(mutations, fmt.Sprintf("quantity = $%v", len(parameters)))
	}

	if patch.Price != nil {
		parameters = append(parameters, *patch.Price)
		mutations = append(mutations, fmt.Sprintf("price = $%v", len(parameters)))
	}

	connection := c.connectDb()
	_, err := connection.Exec(fmt.Sprintf(
		"UPDATE cart.cart_items SET %s WHERE cart_id = $1 AND item_id = $2", strings.Join(mutations, ", ")),
		parameters...,
	)
	assert(err)

	rows, err := connection.Query("SELECT item_id, quantity, price FROM cart.cart_items WHERE cart_id = $1 AND item_id = $2",
		int32(c.cartId),
		int32(patch.Id),
	)
	assert(err)
	defer rows.Close()

	if rows.Next() {
		item := CartItem{}
		rows.Scan(&item.Id, &item.Quantity, &item.Price)

		c.responseJson(w, item)
	} else {
		c.responseNotFound(w)
	}
}

func (c *Controller) HandleDeleteCartsItems(w http.ResponseWriter, _ *http.Request, params httprouter.Params) {
	c.fetchCartId(params)
	if c.err != nil {
		c.responseBadRequest(w, c.err)
		return
	}

	result, err := c.connectDb().Exec("DELETE FROM cart.cart_items WHERE cart_id = $1", int32(c.cartId))
	assert(err)

	rowsAffected, err := result.RowsAffected()
	assert(err)

	if rowsAffected == 0 {
		c.responseNotFound(w)
	} else {
		c.responseEmpty(w)
	}
}

func (c *Controller) HandleDeleteCartsItem(w http.ResponseWriter, _ *http.Request, params httprouter.Params) {
	c.fetchCartId(params)
	c.fetchItemId(params)
	if c.err != nil {
		c.responseBadRequest(w, c.err)
		return
	}

	result, err := c.connectDb().Exec("DELETE FROM cart.cart_items WHERE cart_id = $1 AND item_id = $2", int32(c.cartId), int32(c.itemId))
	assert(err)

	rowsAffected, err := result.RowsAffected()
	assert(err)

	if rowsAffected == 0 {
		c.responseNotFound(w)
	} else {
		c.responseEmpty(w)
	}
}

func (c *Controller) fetchCartId(params httprouter.Params) {
	if c.err != nil {
		return
	}

	c.cartId, c.err = strconv.Atoi(params.ByName("cartId"))
}

func (c *Controller) fetchItemId(params httprouter.Params) {
	if c.err != nil {
		return
	}

	c.itemId, c.err = strconv.Atoi(params.ByName("itemId"))
}

func (c *Controller) fetchBody(req *http.Request) {
	if c.err != nil {
		return
	}

	c.body, c.err = io.ReadAll(req.Body)
}

func (c *Controller) connectDb() *connectionWrapper {
	url, err := variables.Get("database_url")
	if err != nil {
		panic(err)
	}

	c.startDbTimer()
	connection := pg.Open(url)
	c.startDbTimer()

	return &connectionWrapper{connection, c}
}

func assert(e error) {
	if e != nil {
		panic(e)
	}
}

func (c *Controller) addProcessingTimeHeader(w http.ResponseWriter) {
	w.Header().Add("X-Processing-Time-Milliseconds", fmt.Sprintf("%v", time.Now().UnixMilli()-c.Start))
	w.Header().Add("X-Database-Time-Milliseconds", fmt.Sprintf("%v", c.accumulatedTimeDb))
}

func (c *Controller) responseNotFound(w http.ResponseWriter) {
	c.addProcessingTimeHeader(w)

	w.WriteHeader(http.StatusNotFound)
	w.Write([]byte("not found"))
}

func (c *Controller) responseBadRequest(w http.ResponseWriter, err error) {
	c.addProcessingTimeHeader(w)

	w.WriteHeader(http.StatusBadRequest)
	w.Write([]byte(err.Error()))
}

func (c *Controller) responseJson(w http.ResponseWriter, data any) {
	c.addProcessingTimeHeader(w)

	json, err := json.Marshal(data)
	assert(err)

	w.Header().Add("Content-Type", "application/json")
	w.WriteHeader(http.StatusOK)
	w.Write(json)
}

func (c *Controller) responseEmpty(w http.ResponseWriter) {
	c.addProcessingTimeHeader(w)

	w.WriteHeader(http.StatusOK)
}
