use spin_sdk::http::{Request, Response};
use spin_sdk::http_component;
use url::Url;
use urlpattern::{UrlPattern, UrlPatternInit};

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
        .body("not found")
        .build())
}

fn handle_route_cart(cart_id: &str, req: Request) -> anyhow::Result<Response> {
    Ok(Response::builder()
        .status(200)
        .header("content-type", "text/plain")
        .body(format!("/carts/:id , id = {}", cart_id))
        .build())
}

fn handle_route_cart_items(cart_id: &str, req: Request) -> anyhow::Result<Response> {
    Ok(Response::builder()
        .status(200)
        .header("content-type", "text/plain")
        .body(format!("/carts/:id/items , id = {}", cart_id))
        .build())
}

fn build_pattern(source: &str) -> UrlPattern {
    let pattern_init = UrlPatternInit {
        pathname: Some(source.to_owned()),
        ..Default::default()
    };

    <UrlPattern>::parse(pattern_init).unwrap()
}

fn handle_route_cart_item(cart_id: &str, item_id: &str, req: Request) -> anyhow::Result<Response> {
    Ok(Response::builder()
        .status(200)
        .header("content-type", "text/plain")
        .body(format!(
            "/carts/:cartId/items/:itemId , cartId = {} , itemId = {}",
            cart_id, item_id
        ))
        .build())
}
