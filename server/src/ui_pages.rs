use crate::*;
use seed::{prelude::*, *};
use std::sync::Arc;

trait Page: std::fmt::Display {
    fn get_path(&self) -> &'static str;
    fn handle_req(&self, state: Arc<State>) -> Node<()>;
    fn view(&self, state: Arc<State>) -> String {
        let heading = h1![self.to_string()];
        let body = self.handle_req(state);
        let html = div![heading, body];
        html.to_string()
    }
}

struct SetSchedule {}

impl std::fmt::Display for SetSchedule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "Set Schedule")
    }
}

impl Page for SetSchedule {
    fn get_path(&self) -> &'static str {
        "set_schedule"
    }
    fn handle_req(&self, state: Arc<State>) -> Node<()> {
        iframe![
            attrs! {At::Src => "/html/iframe/set_schedule"},
            style! {St::Border => "None", St::Width => "100%"}
        ]
    }
}

const PAGES: &[&dyn Page] = &[&SetSchedule {}];

use warp::{filters::BoxedFilter, Filter, Reply};

fn set_schedule_frame(state: Arc<State>) -> String {
    let guard = state.scheduler.lock().unwrap();
    let rounds = guard.1;
    let tables = &guard.0;
    let mut nodes: Vec<Node<()>> = vec![td![div!["Rounds"]]];
    for (i, table) in tables.iter().enumerate() {
        nodes.push(td![form![
            attrs! {At::Action => format!("/html/iframe/set_schedule/remove/{}", i)},
            table,
            br![],
            button![attrs! {At::Action => "submit"}, "X"]
        ]]);
    }
    nodes.push(td![form![
        attrs! {At::Action => "/html/iframe/set_schedule/add"},
        input![attrs! {At::Name => "add"}],
        button![attrs! {At::Action => "submit"}, "Add"]
    ]]);
    div![table![tr![nodes]]].to_string()
}

#[derive(serde::Deserialize)]
struct AddArg {
    add: usize,
}

fn set_schedule_frame_add(state: Arc<State>, add: AddArg) -> String {
    println!("Adding {}", add.add);
    state.scheduler.lock().unwrap().0.push(add.add);
    set_schedule_frame(state)
}

fn set_schedule_frame_remove(state: Arc<State>, param: usize) -> String {
    println!("Removing {}", param);
    {
        let tables: &mut Vec<_> = &mut state.scheduler.lock().unwrap().0;
        if param < tables.len() {
            tables.remove(param);
        }
    }
    set_schedule_frame(state)
}

pub fn get_html_filter(state: Arc<State>) -> BoxedFilter<(impl Reply,)> {
    let state2 = state.clone();
    let add_filter = warp::path("add")
        .and(warp::query())
        .map(move |table_size| set_schedule_frame_add(state2.clone(), table_size));
    let state2 = state.clone();
    let remove_filter = warp::path("remove")
        .and(warp::path::param())
        .map(move |param: usize| set_schedule_frame_remove(state2.clone(), param));

    let state2 = state.clone();
    let iframe = warp::path("iframe")
        .and(
            warp::path("set_schedule").and(
                add_filter
                    .or(remove_filter)
                    .or(warp::any().map(move || set_schedule_frame(state2.clone()))),
            ),
        )
        .boxed();
    warp::path("html")
        .and(
            warp::path("set_schedule")
                .map(move || SetSchedule {}.view(state.clone()))
                .or(iframe),
        )
        .with(warp::reply::with::header("content-type", "text/html"))
        .boxed()
}
