pub mod database;
pub mod schedule;

extern crate serde;
extern crate serde_json;
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
    pub players: Vec<u32>,
    pub add_player_select_box: String,
}

struct ManagePlayers {
    pub add_player_name_input: String,
}

struct Model {
    pub page: Page,
    pub generate_schedule: GenerateSchedule,
    pub manage_players: ManagePlayers,
    database: database::Database,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            page: Page::GenerateSchedule,
            generate_schedule: GenerateSchedule {
                players: Vec::new(),
                add_player_select_box: String::new(),
            },
            manage_players: ManagePlayers {
                add_player_name_input: String::new(),
            },
            database: database::Database::load(),
        }
    }
}

#[derive(Clone)]
enum Msg {
    ChangePage(Page),
    GSAddPlayer,
    GSAddPlayerSelectBoxInput(String),
    GSRemovePlayer(u32),
    MPAddPlayer,
    MPAddPlayerNameInput(String),
    MPRemovePlayer(u32)
}

fn update(msg: Msg, model: &mut Model, _: &mut impl Orders<Msg>) {
    match msg {
        Msg::ChangePage(page) => {
            model.page = page;
            model.manage_players.add_player_name_input = String::new();
        }
        Msg::GSAddPlayerSelectBoxInput(id) => {
            model.generate_schedule.add_player_select_box = id;
        }
        Msg::GSAddPlayer => {
            let player_id = &model.generate_schedule.add_player_select_box;
            if !player_id.is_empty() {
                if let Ok(id) = player_id.parse::<u32>() {
                    if model.database.contains_player(id) {
                        model.generate_schedule.players.push(id);
                    } else {
                        log!("Played with specified ID does not exist");
                    }
                } else {
                    log!("Invalid ID of player");
                }
            }
        }
        Msg::GSRemovePlayer(id) => {
            if let Some((pos, player)) = model
                .generate_schedule
                .players
                .iter()
                .enumerate()
                .find(|(_, &player_id)| id == player_id)
            {
                model.generate_schedule.players.remove(pos);
            } else {
                log!("Played with specified ID not in list");
            }
        }
        Msg::MPAddPlayerNameInput(player_name) => {
            model.manage_players.add_player_name_input = player_name;
        }
        Msg::MPAddPlayer => {
            let player_name = &model.manage_players.add_player_name_input;
            if !player_name.is_empty() {
                model.database.add_player(player_name.to_string());
                model.manage_players.add_player_name_input = String::new();
            }
        }
        Msg::MPRemovePlayer(id) => {
        	model.database.remove_player(id);
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
            h2!["Tournament Players"],
            p![span!["Group: "], select![], button!["Add"],],
            p![
                span!["Individual: "],
                select![
                    attrs! {At::Value => ""},
                    input_ev(Ev::Input, Msg::GSAddPlayerSelectBoxInput),
                    {
                        let player_list = model.database.get_players();
                        let mut node_list: Vec<Node<Msg>> =
                            Vec::with_capacity(player_list.len() + 1);
                        node_list.push(option![attrs! {At::Value => ""}, ""]);
                        for (id, player) in &player_list {
                            node_list.push(option![
                                attrs! {At::Value => id},
                                format!("{}: ({})", player.name, id)
                            ]);
                        }
                        node_list
                    }
                ],
                button![simple_ev(Ev::Click, Msg::GSAddPlayer), "Add"],
            ],
            ul![style![St::PaddingBottom => "5px";], {
                let mut players_list: Vec<Node<Msg>> =
                    Vec::with_capacity(model.generate_schedule.players.len());
                for &player_id in &model.generate_schedule.players {
                    players_list.push(li![
                        if let Some(player) = model.database.get_player(player_id) {
                            &player.name
                        } else {
                            "Player does not exist"
                        },
                        button![
                            raw_ev(Ev::Click, move |_| Msg::GSRemovePlayer(player_id)),
                            "Remove"
                        ]
                    ]);
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

fn view_manage_players(model: &Model) -> Node<Msg> {
    let box_style = style![St::PaddingLeft => "15px";
St::PaddingRight => "15px";
St::FlexGrow=> "1";];

    div![
        style![St::Display => "Flex";
        St::FlexWrap => "Wrap"],
        div![
            &box_style,
            h2!["Player List"],
            ul![style![St::PaddingBottom => "5px";], {
                let player_list = model.database.get_players();
                let mut node_list: Vec<Node<Msg>> = Vec::with_capacity(player_list.len());
                for (&id, player) in &player_list {
                    node_list.push(li![player.name, button![
                    	raw_ev(Ev::Click, move |_| Msg::MPRemovePlayer(id)),
                            "Remove"
                    ]]);
                }
                node_list
            }],
        ],
        div![
            &box_style,
            p![
                span!["Player Name: "],
                input![input_ev(Ev::Input, Msg::MPAddPlayerNameInput)],
            ],
            button![simple_ev(Ev::Click, Msg::MPAddPlayer), "Add"],
        ],
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
            Page::ManagePlayers => view_manage_players(model),
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
