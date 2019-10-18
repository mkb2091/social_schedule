pub mod schedule;
#[macro_use]
extern crate seed;
use seed::prelude::*;

#[derive(Clone)]
enum Page {
	GenerateSchedule,
	ManagePlayers,
	ManageGroups
}

struct Model {
    pub val: i32,
    pub page: Page
}

impl Default for Model {
    fn default() -> Self {
        Self {
            val: 0,
            page: Page::GenerateSchedule
        }
    }
}


// Update

#[derive(Clone)]
enum Msg {
	ChangePage(Page),
    Increment,
}

fn update(msg: Msg, model: &mut Model, _: &mut impl Orders<Msg>) {
    match msg {
        Msg::Increment => model.val += 1,
        Msg::ChangePage(page) => model.page = page,
    }
}


// View

fn view(model: &Model) -> impl View<Msg> {
vec![
title![format!("{}", match model.page {
    Page::GenerateSchedule => "Generate Schedule",
    Page::ManagePlayers => "Manage Players",
    Page::ManageGroups => "Manage Groups",
    })],
    h1![format!("{}", match model.page {
    Page::GenerateSchedule => "Generate Schedule",
    Page::ManagePlayers => "Manage Players",
    Page::ManageGroups => "Manage Groups",
    })],
    button![
        simple_ev(Ev::Click, Msg::ChangePage(Page::GenerateSchedule)),
        "Generate Schedule"
    ],
    button![
        simple_ev(Ev::Click, Msg::ChangePage(Page::ManagePlayers)),
        "Manage Players"
    ],
    button![
        simple_ev(Ev::Click, Msg::ChangePage(Page::ManageGroups)),
        "Manage Groups"
    ],]
    
    
}

#[wasm_bindgen(start)]
pub fn render() {
    seed::App::build(|_, _| Model::default(), update, view)
        .finish()
        .run();

}
