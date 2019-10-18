pub mod schedule;
#[macro_use]
extern crate seed;
use seed::prelude::*;

#[derive(Clone)]
enum Page {
    GenerateSchedule,
    ManagePlayers,
    ManageGroups,
}

struct GenerateSchedule {
    pub players: Vec<String>,
    pub add_player_input: String,
}

struct Model {
    pub page: Page,
    pub generate_schedule: GenerateSchedule,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            page: Page::GenerateSchedule,
            generate_schedule: GenerateSchedule {
                players: vec![],
                add_player_input: String::new(),
            },
        }
    }
}

#[derive(Clone)]
enum Msg {
    ChangePage(Page),
    GSAddPlayer,
    GSAddPlayerInput(String),
}

fn update(msg: Msg, model: &mut Model, _: &mut impl Orders<Msg>) {
    match msg {
        Msg::ChangePage(page) => model.page = page,
        Msg::GSAddPlayerInput(player) => {
            model.generate_schedule.add_player_input = player;
        }
        Msg::GSAddPlayer => {
            let player = &model.generate_schedule.add_player_input;
            if !player.is_empty() {
                model.generate_schedule.players.push(player.clone());
                model.generate_schedule.add_player_input = String::new();
            }
        }
    }
}

fn view_generate_schedule(model: &Model) -> Node<Msg> {
    div![div![
        h2!["Player list"],
        ul![{
            let mut players_list: Vec<Node<Msg>> =
                Vec::with_capacity(model.generate_schedule.players.len());
            for player in &model.generate_schedule.players {
                players_list.push(li![player, button!["Remove"]]);
            }
            players_list
        }],
        span!["Player Name: "],
        input![input_ev(Ev::Input, Msg::GSAddPlayerInput)],
        button![simple_ev(Ev::Click, Msg::GSAddPlayer), "Add"],
    ]]
}

fn view(model: &Model) -> impl View<Msg> {
    vec![
        title![match model.page {
            Page::GenerateSchedule => "Generate Schedule",
            Page::ManagePlayers => "Manage Players",
            Page::ManageGroups => "Manage Groups",
        }],
        h1![match model.page {
            Page::GenerateSchedule => "Generate Schedule",
            Page::ManagePlayers => "Manage Players",
            Page::ManageGroups => "Manage Groups",
        }],
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
        ],
        match model.page {
            Page::GenerateSchedule => view_generate_schedule(model),
            _ => div![],
        },
    ]
}

#[wasm_bindgen(start)]
pub fn render() {
    seed::App::build(|_, _| Model::default(), update, view)
        .finish()
        .run();
}
