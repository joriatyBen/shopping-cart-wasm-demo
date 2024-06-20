use serde::Deserialize;
use spin_sdk::http::{Method, Request, Response};
use spin_sdk::pg::{Connection, DbValue, ParameterValue};
use spin_sdk::{http_component, pg};
use url::Url;
use urlpattern::{UrlPattern, UrlPatternInit};

const DB_URL_ENV: &str = "DB_URL";

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
    let pattern_cart = build_pattern("/carts/:cartId");
    let pattern_cart_items = build_pattern("/carts/:cartId/items");
    let pattern_cart_item = build_pattern("/carts/:cartId/items/:itemId");

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
        Ok(Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&GetCartResponse { id: cart_id })?)
            .build())
    } else {
        Ok(Response::builder()
            .status(404)
            .body("id not found".to_owned())
            .build())
    }
}

fn get_cart_items(cart_id: u32) -> anyhow::Result<Response> {
    let rowset = open_connection().query(
        "SELECT item_id, quantity, price FROM cart.cart_items WHERE cart_id = $1",
        &vec![ParameterValue::Int32(cart_id as i32)],
    )?;

    if rowset.rows.len() == 0 {
        return Ok(Response::builder()
            .status(404)
            .body("id not found".to_owned())
            .build());
    }

    let items = rowset
        .rows
        .iter()
        .map(|row| {
            return CartItem {
                id: db_value_as_int(&row[0]).unwrap() as u32,
                quantity: db_value_as_int(&row[1]).unwrap() as u32,
                price: db_value_as_float(&row[2]).unwrap(),
            };
        })
        .collect::<Vec<CartItem>>();

    Ok(Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&items).unwrap())
        .build())
}

fn post_cart_items(cart_id: u32, req: Request) -> anyhow::Result<Response> {
    let item = parse_json::<CartItem>(req.body());
    if !item.is_ok() {
        return response_bad_request(item.unwrap_err());
    }
    let item = item.unwrap();

    let sql_result = open_connection().execute(
        "INSERT INTO cart.cart_items VALUES($1, $2, $3, $4);",
        &vec![
            ParameterValue::Int32(cart_id as i32),
            ParameterValue::Int32(item.id as i32),
            ParameterValue::Int32(item.quantity as i32),
            ParameterValue::Floating64(item.price),
        ],
    );

    if sql_result.is_err() {
        return response_bad_request(anyhow::Error::msg("duplicate item"));
    }

    Ok(Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&item).unwrap())
        .build())
}

fn patch_cart_items(cart_id: u32, req: Request) -> anyhow::Result<Response> {
    let patch = parse_json::<CartItemPatch>(req.body());
    if !patch.is_ok() {
        return response_bad_request(patch.unwrap_err());
    }

    Ok(Response::builder()
        .status(200)
        .body(format!("Patch cart={} item {:?}", cart_id, patch))
        .build())
}

fn delete_cart_items(cart_id: u32) -> anyhow::Result<Response> {
    let exec_result = open_connection().execute(
        "DELETE FROM cart.cart_items WHERE cart_id = $1 ",
        &vec![ParameterValue::Int32(cart_id as i32)],
    );

    if exec_result.is_err() {
        return Ok(Response::builder()
            .status(500)
            .body(exec_result.unwrap_err().to_string())
            .build());
    }

    Ok(Response::builder().status(200).build())
}

fn delete_cart_item(cart_id: u32, item_id: u32) -> anyhow::Result<Response> {
    let exec_result = open_connection().execute(
        "DELETE FROM cart.cart_items WHERE cart_id = $1 AND item_id = $2",
        &vec![
            ParameterValue::Int32(cart_id as i32),
            ParameterValue::Int32(item_id as i32),
        ],
    );

    if exec_result.is_err() {
        return Ok(Response::builder()
            .status(500)
            .body(exec_result.unwrap_err().to_string())
            .build());
    }

    Ok(Response::builder().status(200).build())
}

fn parse_json<'a, T: Deserialize<'a>>(json: &'a [u8]) -> anyhow::Result<T> {
    let json = std::str::from_utf8(json)?;

    serde_json::from_str(json).map_err(anyhow::Error::msg)
}

fn response_bad_request(e: anyhow::Error) -> anyhow::Result<Response> {
    Ok(Response::builder().status(400).body(e.to_string()).build())
}

fn open_connection() -> Connection {
    let address = std::env::var(DB_URL_ENV).unwrap();

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
