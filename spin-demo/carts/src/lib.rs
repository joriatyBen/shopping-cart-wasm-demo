use spin_sdk::http::{IntoResponse, Request, Response};
use spin_sdk::http_component;
use urlpattern::{UrlPattern, UrlPatternInit};

/// A simple Spin HTTP component.
#[http_component]
fn handle_carts(req: Request) -> anyhow::Result<impl IntoResponse> {
    let patternDefCart = UrlPatternInit {
        pathname: Some("/carts/:id".to_owned()),
        ..Default::default()
    };

    let patternCart = <UrlPattern>::parse(patternDefCart).unwrap();

    let url = "https://example.com/users/123".parse().unwrap();

    println!("Handling request to {:?}", req.header("spin-full-url"));
    Ok(Response::builder()
        .status(200)
        .header("content-type", "text/plain")
        .body("Hello, Fermyon --- /carts/...")
        .build())
}
