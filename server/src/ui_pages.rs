use crate::*;
use seed::{prelude::*, *};
use std::sync::Arc;

use warp::{filters::BoxedFilter, Filter, Reply};

trait Page: std::fmt::Display {
    fn get_path(&self) -> &'static str;
    fn handle_req(&self, state: Arc<State>) -> Node<()>;
    fn view(&self, state: Arc<State>) -> String {
        let title = h1![self.to_string()];
        let mut heading = Vec::new();
        for page in PAGES.iter() {
            heading.push(a![
                attrs! {At::Href => page.get_path()},
                h2![page.to_string()]
            ]);
        }
        let body = self.handle_req(state);
        let html = div![title, heading, body];
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
    fn handle_req(&self, _state: Arc<State>) -> Node<()> {
        iframe![
            attrs! {At::Src => "/html/iframe/set_schedule"},
            style! {St::Border => "None", St::Width => "100%"}
        ]
    }
}

struct Config {}

impl std::fmt::Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "Config")
    }
}

impl Page for Config {
    fn get_path(&self) -> &'static str {
        "config"
    }
    fn handle_req(&self, state: Arc<State>) -> Node<()> {
        form![
            input![
                attrs! {At::Name => "client_buffer_size", At::Value => state.client_buffer_size.load(Ordering::Relaxed)}
            ],
            button![attrs! {At::Action => "submit"}, "Set"]
        ]
    }
}

struct Status {}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "Status")
    }
}

impl Page for Status {
    fn get_path(&self) -> &'static str {
        "status"
    }
    fn handle_req(&self, state: Arc<State>) -> Node<()> {
        fn to_appropriate_unit(bytes: u128) -> String {
            let bytes = byte_unit::Byte::from_bytes(bytes);
            let adjusted_byte = bytes.get_appropriate_unit(false);
            adjusted_byte.to_string()
        }
        let mut nodes = Vec::new();
        let solve_states = state.all_schedule_solve_states();
        for (arg, solve_state) in solve_states.iter() {
            let unclaimed = solve_state.get_unclaimed_len();
            let queue = solve_state.get_queue_len();
            let mut clients = Vec::new();
            let mut total_rate = 0.0;
            let mut total_recieved_rate = 0.0;
            let mut total_sent_rate = 0.0;
            for client in solve_state.get_clients().iter() {
                let rate = client.get_rate();
                total_rate += rate;
                let recieved_rate = client.get_recieved_rate();
                total_recieved_rate += recieved_rate;
                let sent_rate = client.get_sent_rate();
                total_sent_rate += sent_rate;
                clients.push(tr![
                    td![client.get_id().to_string()],
                    td![client.claimed_len(), " claimed"],
                    td![rate.to_string(), " ops/s"],
                    td![to_appropriate_unit(recieved_rate as u128), " /s"],
                    td![to_appropriate_unit(sent_rate as u128), " /s"],
                ]);
            }
            let total = tr![
                td!["Total"],
                td![],
                td![total_rate.to_string(), " ops/s"],
                td![to_appropriate_unit(total_recieved_rate as u128), " /s"],
                td![to_appropriate_unit(total_sent_rate as u128), " /s"],
            ];
            let node: Node<()> = div![format!(
                "{:?}: {} unclaimed, {} in queue, total rate: {} steps/s",
                arg, unclaimed, queue, total_rate
            )];
            let heading = tr![
                td!["Client"],
                td!["Claimed"],
                td!["Rate"],
                td!["Recieved"],
                td!["Sent"],
            ];
            nodes.push(div![node, table![heading, total, clients]]);
        }
        div![nodes]
    }
}

const PAGES: &[&dyn Page] = &[&SetSchedule {}, &Status {}, &Config {}];

fn set_schedule_frame(state: Arc<State>) -> String {
    let guard = state.scheduler.lock().unwrap();
    let rounds = guard.1;
    let tables = &guard.0;
    let mut nodes: Vec<Node<()>> = vec![td![div!["Tables"]]];
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
    let rounds = form![
        attrs! {At::Action => "/html/iframe/set_schedule/set_rounds"},
        input![attrs! {At::Name => "rounds", At::Value => rounds}],
        button![attrs! {At::Action => "submit"}, "Change number of rounds"]
    ];
    let html = div![table![tr![nodes]], rounds];

    html.to_string()
}

#[derive(serde::Deserialize)]
struct AddArg {
    add: usize,
}

fn set_schedule_frame_add(state: Arc<State>, arg: AddArg) -> String {
    println!("Adding {}", arg.add);
    state.scheduler.lock().unwrap().0.push(arg.add);
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

#[derive(serde::Deserialize)]
struct SetRoundsArg {
    rounds: usize,
}

fn set_schedule_frame_set_rounds(state: Arc<State>, arg: SetRoundsArg) -> String {
    println!("Setting rounds: {}", arg.rounds);
    state.scheduler.lock().unwrap().1 = arg.rounds;
    set_schedule_frame(state)
}

#[derive(serde::Deserialize)]
struct SetConfig {
    client_buffer_size: usize,
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
    let set_rounds_filter = warp::path("set_rounds")
        .and(warp::query())
        .map(move |param: SetRoundsArg| set_schedule_frame_set_rounds(state2.clone(), param));
    let state2 = state.clone();
    let state3 = state.clone();
    let config_filter = warp::path("config").and(
        warp::query()
            .map(move |param: SetConfig| {
                state3
                    .client_buffer_size
                    .store(param.client_buffer_size, Ordering::Relaxed);
                Config {}.view(state3.clone())
            })
            .or(warp::any().map(move || Config {}.view(state2.clone()))),
    );

    let state2 = state.clone();
    let iframe = warp::path("iframe")
        .and(
            warp::path("set_schedule").and(
                add_filter
                    .or(remove_filter)
                    .or(set_rounds_filter)
                    .or(warp::any().map(move || set_schedule_frame(state2.clone()))),
            ),
        )
        .boxed();
    let state2 = state.clone();
    warp::path("html")
        .and(
            warp::path("set_schedule")
                .map(move || SetSchedule {}.view(state2.clone()))
                .or(warp::path("status").map(move || Status {}.view(state.clone())))
                .or(config_filter)
                .or(iframe),
        )
        .with(warp::reply::with::header("content-type", "text/html"))
        .boxed()
}
