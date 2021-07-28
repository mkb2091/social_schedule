use std::sync::Arc;

use warp::Filter;

use server::*;

fn favicon() -> &'static [u8] {
    &[]
}

#[tokio::main]
async fn main() {
    let state = Arc::new(State::new());

    let favicon = warp::path("favicon.ico").map(favicon);
    let html = ui_pages::get_html_filter(state.clone());
    let api = api::get_api_filter(state.clone());
    println!("Server Launched");

    warp::serve(favicon.or(html).or(api))
        .run(([0, 0, 0, 0], 3000))
        .await;
}
