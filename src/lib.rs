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
    Preferences,
}

struct Model {
    pub page: Page,
    pub generate_schedule: generate_schedule_page::GenerateSchedule,
    pub manage_players: manage_players_page::ManagePlayers,
    manage_groups: manage_groups_page::ManageGroups,
    manage_events: manage_events_page::ManageEvents,
    create_event: manage_events_page::CreateEvent,
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
    GSMakeEvent,
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
        Msg::GSGenerate => model.generate_schedule.generate(),
        Msg::GSMakeEvent => model.generate_schedule.make_event(&mut model.database),
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
    node_list.push(option![style.option_style(), attrs! {At::Value => ""}, ""]);
    for (id, player) in &player_list {
        if !ignored_players.contains(id) || Some(**id) == selected {
            node_list.push(option![
                style.option_style(),
                attrs! {At::Value => id},
                if !ignored_players.contains(id) {
                    format!("{} (ID: {})", player.name, id)
                } else {
                    "".to_string()
                }
            ]);
        }
    }
    node_list
}

fn view(model: &Model) -> impl View<Msg> {
    let tab_style = style![St::FlexGrow => "1";];
    div![
        model.style_control.base_style(),
        style![St::Flex => "1";
St::Overflow => "auto";],
        h1![match model.page {
            Page::ManagePlayers => "Manage Players",
            Page::ManageGroups => "Manage Groups",
            Page::ManageEvents => "Manage Events",
            Page::CreateEvent => "Create Event",
            Page::Preferences => "Preferences",
        }],
        div![
            style![St::Display => "Flex";
		St::FlexWrap => "Wrap";],
            button![
                model.style_control.button_style(),
                &tab_style,
                simple_ev(Ev::Click, Msg::ChangePage(Page::CreateEvent)),
                "Create Event"
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

            Page::Preferences => {
                preferences_page::view_preferences(&model.preferences, &model.style_control)
            }
        },
    ]
}

fn window_events(_model: &Model) -> Vec<seed::events::Listener<Msg>> {
    let mut result = Vec::new();
    result.push(simple_ev(Ev::Playing, Msg::GSGenerate));
    result
}

#[wasm_bindgen(start)]
pub fn render() {
    seed::App::build(|_, _| Init::new(Model::default()), update, view)
        .window_events(window_events)
        .build_and_start();
}
