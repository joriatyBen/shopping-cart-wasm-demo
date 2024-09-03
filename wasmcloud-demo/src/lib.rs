wit_bindgen::generate!({
   generate_all,
   with: {
       "wasi:http/types@0.2.0": ::wasi::http::types,
       "wasi:io/error@0.2.0": ::wasi::io::error,
       "wasi:io/poll@0.2.0": ::wasi::io::poll,
       "wasi:io/streams@0.2.0": ::wasi::io::streams,
   }
});

use std::arch::x86_64;
use std::io::Read;
use std::result::Result::Ok;

use anyhow::{anyhow, bail, ensure, Error};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::from_slice;

use url::Url;
use urlpattern::{UrlPattern, UrlPatternInit, UrlPatternResult};

use ::wasi::http::types::*;
use crate::wasi::logging::logging::{log, Level};

use wasmcloud::postgres::query::query;
use wasmcloud::postgres::types::{PgValue, ResultRow, ResultRowEntry};

use exports::wasi::http::incoming_handler::Guest;
struct HttpServer;

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct CartItem {
    #[serde(rename = "itemId")]
    id: u32,
    quantity: u32,
    price: f64,
}

impl CartItem {
    fn default() -> CartItem {
        CartItem{id: 0, quantity: 0, price: 0.0}
    }

    fn set_id(&mut self, id_value: i32) {
        self.id = id_value as u32;
    }

    fn set_quantity(&mut self, quantity_value: i32) {
        self.quantity = quantity_value as u32;
    }

    fn set_price(&mut self, price_value: f64) {
        self.price = price_value
    }
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

const SELECT_FROM_CART_WHERE_ID: &str = r#"
SELECT * FROM cart.cart_items WHERE cart_id = $1;
"#;

const SELECT_ATTRS_FROM_CART_WHERE_ID: &str = r#"
SELECT item_id, quantity, price FROM cart.cart_items WHERE cart_id = $1;
"#;

const INSERT_INTO_CART_ITEMS: &str = r#"
INSERT INTO cart.cart_items VALUES($1, $2, $3, $4);
"#;

impl Guest for HttpServer {
    fn handle(request: IncomingRequest, response_out: ResponseOutparam) {
        let pattern_cart = build_pattern("/carts-wcrs/:cartId");
        let pattern_cart_items = build_pattern("/carts-wcrs/:cartId/items");
        let pattern_cart_item = build_pattern("/carts-wcrs/:cartId/items/:itemId");

        let path_with_query = request
            .path_with_query()
            .map(String::from)
            .unwrap_or_else(|| "/".into());

        // the "http://..." string serves as placeholder for the url pattern match
        let mut base_url = "http://127.0.0.1:8080".to_owned();
        base_url.push_str(&path_with_query.as_str());
        
        let url = Url::parse(&base_url).expect("Failed to parse URL");

        let match_cart = pattern_cart
            .exec(urlpattern::UrlPatternMatchInput::Url(url.clone()))
            .unwrap();

        if match_cart.is_some() {
          return handle_route_cart(
              match_cart.unwrap().pathname.groups.get("cartId").unwrap(),
              request,
              response_out,
          ).unwrap();
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
                request,
                response_out,
            ).unwrap();
        }
    }
}

// read_to_end(&mut buf) results in timeout
fn read_body(request: IncomingRequest) -> Result<Vec<u8>, ()> {
    let mut buf = vec![];

    let body = request
        .consume()
        .expect("failed to consume request body");

    let mut input_stream = body
        .stream()
        .expect("failed to get stream from request body");

    let x = input_stream
        .read_to_end(&mut buf)
        .expect("failed");
    log(    
        Level::Info,
        "buf",
        format!("{x:#?}").as_str(),
    );

    drop(body);
    Ok(buf)
}

fn build_pattern(source: &str) -> UrlPattern {
    let pattern_init = UrlPatternInit {
        pathname: Some(source.to_owned()),
        ..Default::default()
    };
    <UrlPattern>::parse(pattern_init).unwrap()
}

fn handle_route_cart(cart_id: &str, request: IncomingRequest, response_out: ResponseOutparam) -> Result<(), Error> {
    let cart_id = cart_id.parse::<u32>();
    if !cart_id.is_ok() {
        return Ok(response_bad_request(response_out, cart_id.map_err(anyhow::Error::msg).unwrap_err()));
    }

    let cart_id = cart_id.unwrap() as i32;

    match request.method() {
        Method::Get => Ok(get_cart(cart_id, response_out)),
        _ => Ok(response_not_found(response_out)),
    }
}

fn handle_route_cart_items(cart_id: &str, request: IncomingRequest, response_out: ResponseOutparam) -> Result<(), Error> {
    let cart_id = cart_id.parse::<u32>();
    if !cart_id.is_ok() {
        return Ok(response_bad_request(response_out, cart_id.map_err(anyhow::Error::msg).unwrap_err()));
    }
    let cart_id = cart_id.unwrap() as i32;

    match request.method() {
        Method::Get => Ok(get_cart_items(cart_id, response_out)),
        Method::Post => Ok(post_cart_items(cart_id, response_out, request)),
//       Method::Patch => patch_cart_items(cart_id, req),
//       Method::Delete => delete_cart_items(cart_id),
        _ => Ok(response_not_found(response_out)),
    }
}

fn get_cart(cart_id: i32, response_out: ResponseOutparam) -> () {
    match query(
        SELECT_FROM_CART_WHERE_ID, 
        &[PgValue::Int(cart_id).into()],
    ) {
        Ok(rows) => response_json(Some(&GetCartResponse { id: cart_id as u32 }), None, response_out),
        Err(e) => response_not_found(response_out),
    };
}

fn get_cart_items(cart_id: i32, response_out: ResponseOutparam) -> () {
    match query(
        SELECT_ATTRS_FROM_CART_WHERE_ID, 
        &[PgValue::Int(cart_id).into()],
    ) {
        Ok(rows) => response_json(Some(&cart_item_from_row(&rows)) ,Some(&format!("{rows:#?}")), response_out),
        Err(e) => response_not_found(response_out),
    };
}

fn post_cart_items(cart_id: i32, response_out: ResponseOutparam, request: IncomingRequest) -> () {
    let body_bytes = read_body(request).unwrap();
    
    log(
        Level::Info,
        "body_bytes",
        format!("{body_bytes:#?}").as_str(),
    );

    if body_bytes.is_empty() {
        return create_response::<()>(response_out, 500, None, Some("request body is empty"));

    }

    let item = parse_json::<CartItem>(&body_bytes);
    
    log(
        Level::Info,
        "item",
        format!("{item:#?}").as_str(),
    );
    
    if !item.is_ok() {
        return create_response::<()>(response_out, 500, None, Some("request body error"));
    }
    
    let item = item.expect("Failed to map incoming request body to CartItem predefined struct");
   
    match query(
        INSERT_INTO_CART_ITEMS,
        &[
            PgValue::Int(cart_id as i32),
            PgValue::Int(item.id as i32),
            PgValue::Int(item.quantity as i32),
            PgValue::Float4(decompose_f64_custom(item.price)),
            ]
    ) {
        Ok(rows) => response_json(Some(&item), None, response_out),
        Err(e) => response_bad_request(response_out, anyhow::Error::msg("duplicate item")),
    };
}

fn cart_item_from_row(row: &Vec<Vec<ResultRowEntry>>) -> CartItem {
    let mut cart_item = CartItem::default();
    for result_row in row.iter() {
        for entry in result_row.iter() {
            if entry.column_name == "cart_id" {
                cart_item.set_id(db_value_as_int(&entry.value).unwrap());
            } else if entry.column_name == "quantity" {
                cart_item.set_quantity(db_value_as_int(&entry.value).unwrap());
            }  else if entry.column_name == "price" {
                cart_item.set_price(db_value_as_float(&entry.value).unwrap());
            }
        }
    }
    cart_item
}

fn db_value_as_int(value: &PgValue) -> Result<i32, Error> {
    match value {
        PgValue::Int(v) => Ok(v.clone() as i32),
        PgValue::Int2(v) => Ok(v.clone() as i32),
        PgValue::Int4(v) => Ok(v.clone() as i32),
        PgValue::Int8(v) => Ok(v.clone() as i32),
        PgValue::BigInt(v) => Ok(v.clone() as i32),
        _ =>  Result::Err(anyhow::Error::msg("not an int")),
    }
}


fn db_value_as_float(value: &PgValue) -> Result<f64, Error> {
    match value { 
        PgValue::Float4((mantissa, exponent, _sign)) => {
            let sign = if _sign < &0 { -1.0 } else { 1.0 };
            let float_value = sign * (*mantissa as f64) * (2.0_f64.powi(*exponent as i32));
            Ok(float_value)
        }
        PgValue::Float8((mantissa, exponent, _sign)) => {
            let sign = if _sign < &0 { -1.0 } else { 1.0 };
            let float_value = sign * (*mantissa as f64) * (2.0_f64.powi(*exponent as i32));
            Ok(float_value)
        }
        PgValue::Double((mantissa, exponent, _sign)) => {
            let sign = if _sign < &0 { -1.0 } else { 1.0 };
            let float_value = sign * (*mantissa as f64) * (2.0_f64.powi(*exponent as i32));
            Ok(float_value)
        }
        _ => Result::Err(anyhow::Error::msg("not an float")),
    }
}

fn create_response<T: Serialize>(
    response_out: ResponseOutparam,
    status_code: u16,
    content: Option<&T>,
    default_message: Option<&str>,
) -> () {
    let response = OutgoingResponse::new(Fields::new());
    response.set_status_code(status_code).unwrap();

    let response_body = response.body().unwrap();

    let body_content = if let Some(object) = content {
        serde_json::to_string(object).unwrap()
    } else if let Some(message) = default_message {
        message.to_string()
    } else {
        String::new()
    };
    
    response_body
        .write()
        .unwrap()
        .blocking_write_and_flush(body_content.as_bytes())
        .unwrap();
    
    OutgoingBody::finish(response_body, None).expect("failed to finish response body");
    ResponseOutparam::set(response_out, Ok(response));
}

fn response_json<T: Serialize>(object: Option<&T>, rows: Option<&str>, response_out: ResponseOutparam) -> () {
    create_response(response_out, 200, object, rows);
}

fn response_not_found(response_out: ResponseOutparam) -> () {
    create_response::<()>(response_out, 404, None, Some("not found"));
}


fn response_bad_request(response_out: ResponseOutparam, e: anyhow::Error) -> () {
    create_response::<()>(response_out, 400, None, Some(&e.to_string()));
}

fn parse_json<T: DeserializeOwned>(json: &[u8]) -> anyhow::Result<T> {
    let json = std::str::from_utf8(json)?;

    serde_json::from_str(json).map_err(anyhow::Error::msg)
}

fn decompose_f64_custom(value: f64) -> (u64, i16, i8) {
    let bits = value.to_bits();
    let sign = if (bits >> 63) & 1 == 1 { 1 } else { 0 };
    let exponent = ((bits >> 52) & 0x7FF) as i16 - 1023;
    let mantissa = if exponent == -1023 {
        bits & 0xFFFFFFFFFFFFF
    } else {
        bits & 0xFFFFFFFFFFFFF | 0x10000000000000
    };
    
    (mantissa, exponent, sign)
}

export!(HttpServer);