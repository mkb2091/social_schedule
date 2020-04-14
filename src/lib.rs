pub mod add_match_results_page;
pub mod database;
pub mod generate_schedule_page;
pub mod manage_events_page;
pub mod manage_groups_page;
pub mod manage_players_page;
pub mod preferences_page;
pub mod schedule;
pub mod style_control;

extern crate getrandom;
extern crate rand;
extern crate rand_core;
extern crate rand_xorshift;

extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate seed;
use seed::prelude::*;

extern crate wasm_bindgen;

extern crate num_format;

#[cfg(test)]
#[macro_use]
extern crate quickcheck;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
extern "C" {
    fn prompt(text: &str) -> Option<String>;
}

#[wasm_bindgen]
extern "C" {
    fn next_tick(delay: f64);
}

#[wasm_bindgen]
extern "C" {
    fn performance_now() -> f64;
}

#[derive(Clone)]
pub enum Page {
    CreateEvent,
    ManagePlayers,
    ManageGroups,
    ManageEvents,
    AddMatchResults,
    Preferences,
}

struct Model {
    pub page: Page,
    pub generate_schedule: generate_schedule_page::GenerateSchedule,
    pub manage_players: manage_players_page::ManagePlayers,
    manage_groups: manage_groups_page::ManageGroups,
    manage_events: manage_events_page::ManageEvents,
    create_event: manage_events_page::CreateEvent,
    add_match_results: add_match_results_page::AddMatchResults,
    preferences: preferences_page::Preferences,
    style_control: style_control::StyleControl,
    database: database::Database,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            page: Page::CreateEvent,
            generate_schedule: generate_schedule_page::GenerateSchedule::default(),
            manage_players: manage_players_page::ManagePlayers::default(),
            manage_groups: manage_groups_page::ManageGroups::default(),
            manage_events: manage_events_page::ManageEvents::default(),
            create_event: manage_events_page::CreateEvent::default(),
            add_match_results: add_match_results_page::AddMatchResults::default(),
            preferences: preferences_page::Preferences::default(),
            style_control: style_control::StyleControl::default(),
            database: database::Database::load(),
        }
    }
}

#[derive(Clone)]
pub enum Msg {
    ChangePage(Page),
    CESetEventName(String),
    CESetEventDate(String),
    CEAddPlayer,
    CEAddPlayerSelectBoxInput(String),
    CEAddGroup,
    CEAddGroupSelectBoxInput(String),
    CERemovePlayer(u32),
    CERemoveAllPlayers,
    CESetTables(String),
    CEGenerateSchedule,
    GSSetCpuUsage(String),
    GSStop,
    GSResume,
    GSGenerate,
    GSBack,
    GSMakeEvent,
    MEExpandSchedule(u32),
    MEHideSchedule(u32),
    MEDelete(u32),
    MESetPlayerFilter(String),
    ARExpandSchedule(u32),
    ARHideSchedule(u32),
    ARSetScore(u32, usize, usize, usize, String),
    ARAddMatchResults(u32),
    MPAddPlayer,
    MPAddPlayerNameInput(String),
    MPAddPlayerEmailInput(String),
    MPRemovePlayer(u32),
    MPChangeName(u32),
    MPChangeEmail(u32),
    MGAddGroup,
    MGRemoveGroup(u32),
    MGAddGroupNameInput(String),
    MGAddPlayerInput(u32, String),
    MGAddPlayer(u32),
    MGRemovePlayerFromGroup(u32, u32),
    MGChangeName(u32),
    MGExpand(u32),
    MGHide(u32),
    PSetThemeInput(String),
    PSetTheme,
    PExportDatabase,
    PImportDatabase,
}

fn update(msg: Msg, model: &mut Model, _: &mut impl Orders<Msg>) {
    match msg {
        Msg::ChangePage(page) => {
            model.page = page;
        }
        Msg::CESetEventName(name) => model.create_event.set_event_name(name),
        Msg::CESetEventDate(date) => model.create_event.set_event_date(date),
        Msg::CEAddPlayerSelectBoxInput(id) => {
            model.create_event.set_add_player_select_box_input(id)
        }
        Msg::CEAddPlayer => model.create_event.add_player(&model.database),
        Msg::CEAddGroupSelectBoxInput(id) => model.create_event.set_add_group_select_box_input(id),
        Msg::CEAddGroup => model.create_event.add_group(&model.database),
        Msg::CERemovePlayer(id) => model.create_event.remove_player(id),
        Msg::CERemoveAllPlayers => model.create_event.remove_all_players(),
        Msg::CESetTables(tables) => model.create_event.set_tables(tables),
        Msg::CEGenerateSchedule => model
            .create_event
            .go_to_generate_schedule_page(&mut model.generate_schedule),
        Msg::GSSetCpuUsage(cpu_usage) => model.generate_schedule.set_cpu_usage(cpu_usage),
        Msg::GSStop => model.generate_schedule.stop(),
        Msg::GSResume => model.generate_schedule.resume(),
        Msg::GSGenerate => model.generate_schedule.generate(&model.database),
        Msg::GSBack => model.create_event.back(&mut model.generate_schedule),
        Msg::GSMakeEvent => model.generate_schedule.make_event(&mut model.database),
        Msg::MEExpandSchedule(id) => model.manage_events.expand_schedule(id),
        Msg::MEHideSchedule(id) => model.manage_events.hide_schedule(id),
        Msg::MEDelete(id) => model.manage_events.delete(id, &mut model.database),
        Msg::MESetPlayerFilter(id) => model.manage_events.set_filter_by_player(id),
        Msg::ARExpandSchedule(id) => model.add_match_results.expand_schedule(id),
        Msg::ARHideSchedule(id) => model.add_match_results.hide_schedule(id),
        Msg::ARSetScore(id, round, table, player_number, score) => model
            .add_match_results
            .set_score(id, round, table, player_number, score),
        Msg::ARAddMatchResults(id) => model.add_match_results.add_results(id, &mut model.database),
        Msg::MPAddPlayerNameInput(player_name) => {
            model.manage_players.set_player_name_input(player_name)
        }
        Msg::MPAddPlayerEmailInput(player_email) => {
            model.manage_players.set_player_email_input(player_email)
        }
        Msg::MPAddPlayer => model.manage_players.add_player(&mut model.database),
        Msg::MPRemovePlayer(id) => model.manage_players.remove_player(&mut model.database, id),
        Msg::MPChangeName(id) => model.manage_players.change_name(&mut model.database, id),
        Msg::MPChangeEmail(id) => model.manage_players.change_email(&mut model.database, id),
        Msg::MGAddGroup => model.manage_groups.add_group(&mut model.database),
        Msg::MGRemoveGroup(id) => model.manage_groups.remove_group(&mut model.database, id),
        Msg::MGAddGroupNameInput(group_name) => {
            model.manage_groups.set_add_group_name_input(group_name)
        }
        Msg::MGAddPlayerInput(group_id, player_id) => model
            .manage_groups
            .set_add_player_to_group_input(group_id, player_id),
        Msg::MGAddPlayer(id) => model.manage_groups.add_player(&mut model.database, id),
        Msg::MGRemovePlayerFromGroup(group_id, player_id) => {
            model.database.remove_player_from_group(group_id, player_id)
        }
        Msg::MGChangeName(group_id) => model
            .manage_groups
            .change_name(&mut model.database, group_id),
        Msg::MGExpand(group_id) => model.manage_groups.expand(group_id),
        Msg::MGHide(group_id) => model.manage_groups.hide(group_id),
        Msg::PSetThemeInput(theme) => model.preferences.set_theme_input(theme),
        Msg::PSetTheme => model.preferences.set_theme(&mut model.style_control),
        Msg::PExportDatabase => model.preferences.export_database(&model.database),
        Msg::PImportDatabase => model.preferences.import_database(&mut model.database),
    }
}

fn player_select_box(
    database: &database::Database,
    style: &style_control::StyleControl,
    ignored_players: &std::collections::HashSet<u32>,
    selected: Option<u32>,
) -> Vec<Node<Msg>> {
    let mut player_list = database.get_players();
    player_list.sort_by_key(|(_, player)| &player.name);
    let mut node_list: Vec<Node<Msg>> = Vec::with_capacity(player_list.len() + 1);
    node_list.push(option![
        style.option_style(),
        attrs! {At::Value => ""},
        if selected.is_some() {
            attrs! {}
        } else {
            attrs! {At::Selected => "selected"}
        },
        ""
    ]);
    for (id, player) in &player_list {
        if !ignored_players.contains(id) || Some(**id) == selected {
            node_list.push(option![
                style.option_style(),
                attrs! {At::Value => id},
                if Some(**id) == selected {
                    attrs! {At::Selected => "selected"}
                } else {
                    attrs! {}
                },
                if ignored_players.contains(id) {
                    "".to_string()
                } else {
                    format!("{} (ID: {})", player.name, id)
                }
            ]);
        }
    }
    node_list
}

fn view_schedule<T: schedule::ScheduleStructure>(
    schedule: &T,
    players: &[u32],
    database: &database::Database,
    matches: Option<&[Vec<u32>]>,
) -> Node<Msg> {
    table![style![St::BorderSpacing => "5px 10px"; ], {
        let tables = schedule.get_tables();

        let mut table: Vec<Node<Msg>> = Vec::with_capacity(tables + 1);
        let mut heading: Vec<Node<Msg>> = Vec::with_capacity(tables + 1);
        heading.push(td![]);
        for game in 1..=tables {
            heading.push(th![format!("Table {:}", game)]);
        }
        table.push(tr![heading]);
        for round in 0..tables {
            let matches_in_round = if let Some(matches) = matches {
                Some(&matches[round])
            } else {
                None
            };
            table.push(tr![{
                let mut row: Vec<Node<Msg>> = Vec::with_capacity(tables + 1);
                row.push(td![format!("Round {:}", round + 1)]);
                for table in 0..tables {
                    let current_match = {
                        let mut current_match: Option<std::collections::HashMap<u32, usize>> = None;

                        if let Some(matches_in_round) = matches_in_round {
                            if let Some(&match_id) = matches_in_round.get(table) {
                                if let Some(temp_match) = database.get_match(match_id) {
                                    current_match = Some(
                                        temp_match
                                            .ps
                                            .iter()
                                            .cloned()
                                            .collect::<std::collections::HashMap<u32, usize>>(),
                                    )
                                }
                            }
                        }
                        current_match
                    };
                    row.push(td![{
                        let player_list = schedule.get_players_from_game(round, table);
                        let mut data: Vec<Node<Msg>> = Vec::new();
                        for player_number in player_list {
                            let id = players[player_number];
                            if let Some(player) = database.get_player(id) {
                                data.push(span![
                                    style![St::Display => "Flex";
		St::FlexWrap => "Wrap";],
                                    span![style![St::FlexGrow => "1"], player.name],
                                    span![
                                        style! {St::Width => "3em"; St::FlexGrow => "0", St::PaddingLeft => "2em"},
                                        if let Some(current_match) = &current_match {
                                            if let Some(score) = current_match.get(&id) {
                                                format!("{}", score)
                                            } else {
                                                "".to_string()
                                            }
                                        } else {
                                            "".to_string()
                                        }
                                    ],
                                    br![]
                                ]);
                            } else {
                                data.push(span![format!("Unknown player ID: {}", id)])
                            }
                        }
                        data
                    }]);
                }
                row
            }]);
        }
        table
    }]
}

fn view(model: &Model) -> impl View<Msg> {
    let tab_style = style![St::FlexGrow => "1";];
    div![
        model.style_control.base_style(),
        style![St::Flex => "1";
St::Overflow => "auto";],
        h1![
            style![St::PaddingLeft => "20px";],
            match model.page {
                Page::ManagePlayers => "Manage Players".to_string(),
                Page::ManageGroups => "Manage Groups".to_string(),
                Page::ManageEvents => "Manage Events".to_string(),
                Page::CreateEvent => model.create_event.title(&model.generate_schedule),
                Page::AddMatchResults => "Add Match Results".to_string(),
                Page::Preferences => "Preferences".to_string(),
            }
        ],
        div![
            style![St::Display => "Flex";
		St::FlexWrap => "Wrap";],
            button![
                model.style_control.button_style(),
                &tab_style,
                simple_ev(Ev::Click, Msg::ChangePage(Page::CreateEvent)),
                model.create_event.title(&model.generate_schedule),
            ],
            button![
                model.style_control.button_style(),
                &tab_style,
                simple_ev(Ev::Click, Msg::ChangePage(Page::ManagePlayers)),
                "Manage Players"
            ],
            button![
                model.style_control.button_style(),
                &tab_style,
                simple_ev(Ev::Click, Msg::ChangePage(Page::ManageGroups)),
                "Manage Groups"
            ],
            button![
                model.style_control.button_style(),
                &tab_style,
                simple_ev(Ev::Click, Msg::ChangePage(Page::ManageEvents)),
                "Manage Events"
            ],
            button![
                model.style_control.button_style(),
                &tab_style,
                simple_ev(Ev::Click, Msg::ChangePage(Page::AddMatchResults)),
                "Add Match Results"
            ],
            button![
                model.style_control.button_style(),
                &tab_style,
                simple_ev(Ev::Click, Msg::ChangePage(Page::Preferences)),
                "Preferences"
            ]
        ],
        match model.page {
            Page::CreateEvent => manage_events_page::view_create_event(
                &model.create_event,
                &model.generate_schedule,
                &model.database,
                &model.style_control,
            ),
            Page::ManagePlayers => manage_players_page::view_manage_players(
                &model.manage_players,
                &model.database,
                &model.style_control,
            ),
            Page::ManageGroups => manage_groups_page::view_manage_groups(
                &model.manage_groups,
                &model.database,
                &model.style_control,
            ),
            Page::ManageEvents => manage_events_page::view_manage_events(
                &model.manage_events,
                &model.database,
                &model.style_control,
            ),
            Page::AddMatchResults => add_match_results_page::view_add_match_results(
                &model.add_match_results,
                &model.database,
                &model.style_control,
            ),
            Page::Preferences => {
                preferences_page::view_preferences(&model.preferences, &model.style_control)
            }
        },
    ]
}

fn window_events(
    _model: &Model,
) -> Vec<seed::virtual_dom::event_handler_manager::event_handler::EventHandler<Msg>> {
    let mut result = Vec::new();
    result.push(simple_ev(Ev::Playing, Msg::GSGenerate));
    result
}

#[wasm_bindgen(start)]
pub fn render() {
    seed::app::App::builder(update, view)
        .window_events(window_events)
        .build_and_start();
}
