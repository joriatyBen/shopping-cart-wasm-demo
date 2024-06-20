use std::any;

use serde::Deserialize;
use spin_sdk::http::{Method, Request, Response};
use spin_sdk::http_component;
use url::Url;
use urlpattern::{UrlPattern, UrlPatternInit};

#[derive(serde::Deserialize, Debug)]
struct CartItem {
    #[serde(rename = "itemId")]
    id: String,
    quantity: u32,
    price: f64,
}

#[derive(serde::Deserialize, Debug)]
struct CartItemPatch {
    #[serde(rename = "itemId")]
    id: String,
    quantity: Option<u32>,
    price: Option<f64>,
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
    Ok(Response::builder()
        .status(200)
        .body(format!("Get cart={}", cart_id))
        .build())
}

fn get_cart_items(cart_id: u32) -> anyhow::Result<Response> {
    Ok(Response::builder()
        .status(200)
        .body(format!("Get cart={} items", cart_id))
        .build())
}

fn post_cart_items(cart_id: u32, req: Request) -> anyhow::Result<Response> {
    let item = parse_json::<CartItem>(req.body());
    if !item.is_ok() {
        return response_bad_request(item.unwrap_err());
    }
    let item = item.unwrap();

    Ok(Response::builder()
        .status(200)
        .body(format!("Add cart={} item {:?}", cart_id, item))
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

fn delete_cart_item(cart_id: u32, item_id: u32) -> anyhow::Result<Response> {
    Ok(Response::builder()
        .status(200)
        .body(format!("Delete cart={} item={}", cart_id, item_id))
        .build())
}

fn parse_json<'a, T: Deserialize<'a>>(json: &'a [u8]) -> anyhow::Result<T> {
    let json = std::str::from_utf8(json)?;

    serde_json::from_str(json).map_err(anyhow::Error::msg)
}

fn response_bad_request(e: anyhow::Error) -> anyhow::Result<Response> {
    Ok(Response::builder().status(400).body(e.to_string()).build())
}
