use std::sync::{Arc, Mutex};

use warp::Filter;

use server::*;

fn favicon() -> &'static [u8] {
    &[]
}

#[tokio::main]
async fn main() {
    let state = Arc::new(State {
        scheduler: Mutex::new((vec![], 0)),
    });

    let favicon = warp::path("favicon.ico").map(favicon);
    let html = ui_pages::get_html_filter(state.clone());

    println!("Server Launched");

    warp::serve(favicon.or(html))
        .run(([127, 0, 0, 1], 3000))
        .await;
}
