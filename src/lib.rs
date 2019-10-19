pub mod schedule;
#[macro_use]
extern crate seed;
use seed::prelude::*;

#[derive(Clone)]
enum Page {
    GenerateSchedule,
    ManagePlayers,
    ManageGroups,
    Preferences,
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
    let box_style = style![St::PaddingLeft => "15px";
St::PaddingRight => "15px";
St::FlexGrow=> "1";];

    div![
        style![St::Display => "Flex";
        St::FlexWrap => "Wrap"],
        div![
            &box_style,
            h2!["Player list"],
            p![
                span!["Player Name: "],
                input![input_ev(Ev::Input, Msg::GSAddPlayerInput)],
                button![simple_ev(Ev::Click, Msg::GSAddPlayer), "Add"],
            ],
            p![span!["Group: "], select![], button!["Add"],],
            p![span!["Individual: "], select![], button!["Add"],],
            ul![style![St::PaddingBottom => "5px";], {
                let mut players_list: Vec<Node<Msg>> =
                    Vec::with_capacity(model.generate_schedule.players.len());
                for player in &model.generate_schedule.players {
                    players_list.push(li![player, button!["Remove"]]);
                }
                players_list
            }],
        ],
        div![
            &box_style,
            p![
                span!["Players per table: "],
                select![{
                    let mut table_size_list: Vec<Node<Msg>> = Vec::with_capacity(42);
                    for table_size in 2..43 {
                        table_size_list.push(option![
                            attrs! {At::Value => table_size},
                            format!("{}", table_size)
                        ]);
                    }
                    table_size_list
                }],
            ],
            p![
                span!["Email Players: "],
                input![attrs! {At::Type => "checkbox"}],
            ],
            button!["Generate"]
        ],
        div![
            &box_style,
            p![
                span!["Runtime Limit: "],
                input![attrs! {At::Type => "checkbox"}],
                p![
                    style![St::PaddingLeft => "30px";],
                    span!["Max run time: "],
                    input![],
                    button!["Apply"]
                ]
            ],
            p![
                span!["Maximum CPU usage"],
                select![attrs! {At::Value => "100"}, {
                    let mut cpu_options: Vec<Node<Msg>> = Vec::with_capacity(100);
                    for percent in 0..100 {
                        let percent = 100 - percent;
                        cpu_options.push(option![
                            attrs! {At::Value => percent},
                            format!("{}%", percent)
                        ]);
                    }
                    cpu_options
                }]
            ],
        ]
    ]
}

fn view(model: &Model) -> impl View<Msg> {
    vec![
        title![match model.page {
            Page::GenerateSchedule => "Generate Schedule",
            Page::ManagePlayers => "Manage Players",
            Page::ManageGroups => "Manage Groups",
            Page::Preferences => "Preferences",
        }],
        h1![match model.page {
            Page::GenerateSchedule => "Generate Schedule",
            Page::ManagePlayers => "Manage Players",
            Page::ManageGroups => "Manage Groups",
            Page::Preferences => "Preferences",
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
        button![
            simple_ev(Ev::Click, Msg::ChangePage(Page::Preferences)),
            "Preferences"
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
