wit_bindgen::generate!({
    generate_all
});

use std::collections::HashMap;
use std::default;
//use std::ptr::metadata;

use serde::{Deserialize, Serialize};
use serde_json::from_slice;
use url::Url;
use urlpattern::{UrlPattern, UrlPatternInit, UrlPatternResult};

use exports::wasi::http::incoming_handler::Guest;
use wasi::http::types::*;

use wasmcloud::postgres::query::query;
use wasmcloud::postgres::types::{PgValue, ResultRow, ResultRowEntry};

//use exports::wasmcloud::examples::invoke::Guest;

struct HttpServer;

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

const SELECT_FROM_CART_WHERE_ID: &str = r#"
SELECT * FROM cart.cart_items WHERE cart_id = $1;
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

        let mut base_url = "http://127.0.0.1:8080".to_owned();
        base_url.push_str(&path_with_query.as_str());
        
        let url = Url::parse(&base_url).expect("Failed to parse URL");

        let match_cart = pattern_cart
            .exec(urlpattern::UrlPatternMatchInput::Url(url.clone()))
            .unwrap();
    
        
        //test_response(response_out, match_cart);

        if match_cart.is_some() {
          return handle_route_cart(
              match_cart.unwrap().pathname.groups.get("cartId").unwrap(),
              request,
              response_out,
          ).unwrap();
      }

    }
}

fn test_response(response_out: ResponseOutparam, id: i32) {
    let response = OutgoingResponse::new(Fields::new());
    response.set_status_code(201).unwrap();
    let response_body = response.body().unwrap();

    response_body
        .write()
        .unwrap()
        .blocking_write_and_flush(format!("{:?}", id).as_bytes())
        .unwrap();
    OutgoingBody::finish(response_body, None).expect("failed to finish response body");
    ResponseOutparam::set(response_out, Ok(response)); 
}

fn build_pattern(source: &str) -> UrlPattern {
    let pattern_init = UrlPatternInit {
        pathname: Some(source.to_owned()),
        ..Default::default()
    };

    <UrlPattern>::parse(pattern_init).unwrap()
}

fn handle_route_cart(cart_id: &str, request: IncomingRequest, response_out: ResponseOutparam) -> Result<(), ()> {
    let cart_id = cart_id.parse::<u32>();
    let outgoing_response = OutgoingResponse::new(Fields::new());
    outgoing_response.set_status_code(400).unwrap();
    if !cart_id.is_ok() {
        return Ok(ResponseOutparam::set(response_out, Ok(outgoing_response)));
    }

    let cart_id = cart_id.unwrap() as i32;

    match request.method() {
        Method::Get => Ok(get_cart(cart_id, response_out)),
        _ => Ok(ResponseOutparam::set(response_out, Ok(outgoing_response))),
    }
}

fn get_cart(cart_id: i32, response_out: ResponseOutparam) -> () {
    match query(
        SELECT_FROM_CART_WHERE_ID, 
        &[PgValue::Int(cart_id).into()],
    ) {
        //Ok(rows) => response_json(&GetCartResponse { id: cart_id as u32 }, response_out),
        Ok(rows) => response_json(&GetCartResponse { id: cart_id as u32 },format!("{rows:#?}"), response_out),
        Err(e) => response_not_found(response_out),
    };
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

// Todo: find a better way to handle retrieved database rows
//   let body_content = if let Some(object) = content {
//       serde_json::to_string(object).unwrap()
//   } else if let Some(message) = default_message {
//       message.to_string()
//   } else {
//       String::new()
//   };
    
    response_body
        .write()
        .unwrap()
        .blocking_write_and_flush(default_message.unwrap().as_bytes())
        .unwrap();
    
    OutgoingBody::finish(response_body, None).expect("failed to finish response body");
    ResponseOutparam::set(response_out, Ok(response));
}

fn response_json<T: Serialize>(object: &T, rows: String, response_out: ResponseOutparam) -> () {
    create_response(response_out, 200, Some(object), Some(&rows));
}

fn response_not_found(response_out: ResponseOutparam) -> () {
    create_response::<()>(response_out, 404, None, Some("not found"));
}


export!(HttpServer);
