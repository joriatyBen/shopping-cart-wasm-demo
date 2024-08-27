use std::time::SystemTime;

use serde::{Deserialize, Serialize};
use spin_sdk::http::{Method, Request, Response};
use spin_sdk::pg::{Connection, DbValue, ParameterValue};
use spin_sdk::{http_component, pg, variables};
use url::Url;
use urlpattern::{UrlPattern, UrlPatternInit};

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct CartItem {
    #[serde(rename = "itemId")]
    id: u32,
    quantity: u32,
    price: f64,
}

#[derive(serde::Deserialize, Debug)]
struct CartItemPatch {
    #[serde(rename = "itemId")]
    id: u32,
    quantity: Option<u32>,
    price: Option<f64>,
}

#[derive(serde::Serialize, Debug)]
struct GetCartResponse {
    #[serde(rename = "customerId")]
    id: u32,
}

#[http_component]
fn handle(req: Request) -> anyhow::Result<Response> {
    let now = SystemTime::now();
    let res = do_handle(req);

    if res.is_ok() {
        let mut res = res.unwrap();
        res.set_header(
            "X-Processing-Time-Milliseconds",
            format!("{}", now.elapsed()?.as_millis()),
        );

        Ok(res)
    } else {
        res
    }
}

fn do_handle(req: Request) -> anyhow::Result<Response> {
    let pattern_cart = build_pattern("/carts-rs/:cartId");
    let pattern_cart_items = build_pattern("/carts-rs/:cartId/items");
    let pattern_cart_item = build_pattern("/carts-rs/:cartId/items/:itemId");

    let url = req
        .header("spin-full-url")
        .and_then(|u| u.as_str())
        .and_then(|u| Url::parse(u).ok())
        .unwrap();

    let match_cart = pattern_cart
        .exec(urlpattern::UrlPatternMatchInput::Url(url.clone()))
        .unwrap();
    if match_cart.is_some() {
        return handle_route_cart(
            match_cart.unwrap().pathname.groups.get("cartId").unwrap(),
            req,
        );
    }

    let match_cart_items = pattern_cart_items
        .exec(urlpattern::UrlPatternMatchInput::Url(url.clone()))
        .unwrap();
    if match_cart_items.is_some() {
        return handle_route_cart_items(
            match_cart_items
                .as_ref()
                .unwrap()
                .pathname
                .groups
                .get("cartId")
                .unwrap(),
            req,
        );
    }

    let match_cart_item = pattern_cart_item
        .exec(urlpattern::UrlPatternMatchInput::Url(url.clone()))
        .unwrap();
    if match_cart_item.is_some() {
        return handle_route_cart_item(
            match_cart_item
                .as_ref()
                .unwrap()
                .pathname
                .groups
                .get("cartId")
                .unwrap(),
            match_cart_item
                .as_ref()
                .unwrap()
                .pathname
                .groups
                .get("itemId")
                .unwrap(),
            req,
        );
    }

    Ok(Response::builder()
        .status(404)
        .header("content-type", "text/plain")
        .build())
}

fn build_pattern(source: &str) -> UrlPattern {
    let pattern_init = UrlPatternInit {
        pathname: Some(source.to_owned()),
        ..Default::default()
    };

    <UrlPattern>::parse(pattern_init).unwrap()
}

fn handle_route_cart(cart_id: &str, req: Request) -> anyhow::Result<Response> {
    let cart_id = cart_id.parse::<u32>();
    if !cart_id.is_ok() {
        return Ok(Response::builder().status(400).build());
    }

    let cart_id = cart_id.unwrap();

    match req.method() {
        Method::Get => get_cart(cart_id),
        _ => Ok(Response::builder().status(400).build()),
    }
}

fn handle_route_cart_items(cart_id: &str, req: Request) -> anyhow::Result<Response> {
    let cart_id = cart_id.parse::<u32>();
    if !cart_id.is_ok() {
        return response_bad_request(cart_id.map_err(anyhow::Error::msg).unwrap_err());
    }
    let cart_id = cart_id.unwrap();

    match req.method() {
        Method::Get => get_cart_items(cart_id),
        Method::Post => post_cart_items(cart_id, req),
        Method::Patch => patch_cart_items(cart_id, req),
        Method::Delete => delete_cart_items(cart_id),
        _ => Ok(Response::builder().status(400).build()),
    }
}

fn handle_route_cart_item(cart_id: &str, item_id: &str, req: Request) -> anyhow::Result<Response> {
    let cart_id = cart_id.parse::<u32>();
    if !cart_id.is_ok() {
        return response_bad_request(cart_id.map_err(anyhow::Error::msg).unwrap_err());
    }
    let cart_id = cart_id.unwrap();

    let item_id = item_id.parse::<u32>();
    if !item_id.is_ok() {
        return response_bad_request(item_id.map_err(anyhow::Error::msg).unwrap_err());
    }
    let item_id = item_id.unwrap();

    match req.method() {
        Method::Delete => delete_cart_item(cart_id, item_id),
        _ => Ok(Response::builder().status(400).build()),
    }
}

fn get_cart(cart_id: u32) -> anyhow::Result<Response> {
    let rowset = open_connection().query(
        "SELECT * FROM cart.cart_items WHERE cart_id = $1;",
        &vec![ParameterValue::Int32(cart_id as i32)],
    )?;

    if rowset.rows.len() > 0 {
        response_json(&GetCartResponse { id: cart_id })
    } else {
        response_not_found()
    }
}

fn get_cart_items(cart_id: u32) -> anyhow::Result<Response> {
    let rowset = open_connection().query(
        "SELECT item_id, quantity, price FROM cart.cart_items WHERE cart_id = $1",
        &vec![ParameterValue::Int32(cart_id as i32)],
    )?;

    if rowset.rows.len() == 0 {
        return response_not_found();
    }

    let items = rowset
        .rows
        .iter()
        .map(cart_item_from_row)
        .collect::<Vec<CartItem>>();

    response_json(&items)
}

fn post_cart_items(cart_id: u32, req: Request) -> anyhow::Result<Response> {
    let item = parse_json::<CartItem>(req.body());
    if !item.is_ok() {
        return response_bad_request(item.unwrap_err());
    }
    let item = item.unwrap();

    let insert_result = open_connection().execute(
        "INSERT INTO cart.cart_items VALUES($1, $2, $3, $4);",
        &vec![
            ParameterValue::Int32(cart_id as i32),
            ParameterValue::Int32(item.id as i32),
            ParameterValue::Int32(item.quantity as i32),
            ParameterValue::Floating64(item.price),
        ],
    );

    if insert_result.is_err() {
        return response_bad_request(anyhow::Error::msg("duplicate item"));
    }

    response_json(&item)
}

fn patch_cart_items(cart_id: u32, req: Request) -> anyhow::Result<Response> {
    let patch = parse_json::<CartItemPatch>(req.body());
    if !patch.is_ok() {
        return response_bad_request(patch.unwrap_err());
    }
    let patch = patch.unwrap();

    let mut mutations = Vec::<String>::with_capacity(2);
    let mut query_parameters = vec![
        ParameterValue::Int32(cart_id as i32),
        ParameterValue::Int32(patch.id as i32),
    ];

    if patch.quantity.is_some() {
        query_parameters.push(ParameterValue::Int32(patch.quantity.unwrap() as i32));
        mutations.push(format!("quantity = ${}", query_parameters.len()));
    }

    if patch.price.is_some() {
        query_parameters.push(ParameterValue::Floating64(patch.price.unwrap() as f64));
        mutations.push(format!("price = ${}", query_parameters.len()));
    }

    let connection = open_connection();

    if mutations.len() > 0 {
        let exec_result = connection.execute(
            &format!(
                "UPDATE cart.cart_items SET {} WHERE cart_id = $1 AND item_id = $2",
                mutations.join(", ")
            ),
            &query_parameters,
        );

        if exec_result.is_err() {
            return response_internal_server_error(
                exec_result.map_err(anyhow::Error::msg).unwrap_err(),
            );
        }
    }

    let rowset = connection.query(
        "SELECT item_id, quantity, price FROM cart.cart_items WHERE cart_id = $1 AND item_id = $2",
        &vec![
            ParameterValue::Int32(cart_id as i32),
            ParameterValue::Int32(patch.id as i32),
        ],
    )?;

    if rowset.rows.len() == 0 {
        return response_not_found();
    }

    response_json(&cart_item_from_row(&rowset.rows[0]))
}

fn delete_cart_items(cart_id: u32) -> anyhow::Result<Response> {
    let query_result = open_connection().query(
        "DELETE FROM cart.cart_items WHERE cart_id = $1 RETURNING *",
        &vec![ParameterValue::Int32(cart_id as i32)],
    )?;

    if query_result.rows.len() == 0 {
        response_not_found()
    } else {
        response_empty()
    }
}

fn delete_cart_item(cart_id: u32, item_id: u32) -> anyhow::Result<Response> {
    let query_result = open_connection().query(
        "DELETE FROM cart.cart_items WHERE cart_id = $1 AND item_id = $2 RETURNING *",
        &vec![
            ParameterValue::Int32(cart_id as i32),
            ParameterValue::Int32(item_id as i32),
        ],
    )?;

    if query_result.rows.len() == 0 {
        response_not_found()
    } else {
        response_empty()
    }
}

fn parse_json<'a, T: Deserialize<'a>>(json: &'a [u8]) -> anyhow::Result<T> {
    let json = std::str::from_utf8(json)?;

    serde_json::from_str(json).map_err(anyhow::Error::msg)
}

fn response_bad_request(e: anyhow::Error) -> anyhow::Result<Response> {
    Ok(Response::builder().status(400).body(e.to_string()).build())
}

fn response_internal_server_error(e: anyhow::Error) -> anyhow::Result<Response> {
    Ok(Response::builder().status(500).body(e.to_string()).build())
}

fn response_not_found() -> anyhow::Result<Response> {
    Ok(Response::builder()
        .status(404)
        .body("not found".to_owned())
        .build())
}

fn response_json<T: Serialize>(object: &T) -> anyhow::Result<Response> {
    Ok(Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&object).unwrap())
        .build())
}

fn response_empty() -> anyhow::Result<Response> {
    Ok(Response::builder().status(200).build())
}

fn open_connection() -> Connection {
    let address = variables::get("database_url").unwrap();

    pg::Connection::open(&address).unwrap()
}

fn db_value_as_int(value: &DbValue) -> anyhow::Result<i32> {
    match value {
        DbValue::Int64(x) => Ok(x.clone() as i32),
        DbValue::Int32(x) => Ok(x.clone()),
        DbValue::Int16(x) => Ok(x.clone() as i32),
        DbValue::Int8(x) => Ok(x.clone() as i32),
        DbValue::Uint64(x) => Ok(x.clone() as i32),
        DbValue::Uint32(x) => Ok(x.clone() as i32),
        DbValue::Uint16(x) => Ok(x.clone() as i32),
        DbValue::Uint8(x) => Ok(x.clone() as i32),
        _ => Result::Err(anyhow::Error::msg("not an int")),
    }
}

fn db_value_as_float(value: &DbValue) -> anyhow::Result<f64> {
    match value {
        DbValue::Floating32(x) => Ok(x.clone() as f64),
        DbValue::Floating64(x) => Ok(x.clone()),
        _ => Result::Err(anyhow::Error::msg("not a float")),
    }
}

fn cart_item_from_row(row: &Vec<DbValue>) -> CartItem {
    CartItem {
        id: db_value_as_int(&row[0]).unwrap() as u32,
        quantity: db_value_as_int(&row[1]).unwrap() as u32,
        price: db_value_as_float(&row[2]).unwrap(),
    }
}
