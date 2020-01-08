pub mod database;
pub mod generate_schedule_page;
pub mod manage_groups_page;
pub mod manage_players_page;
pub mod preferences_page;
pub mod schedule;
pub mod style_control;

extern crate rand_core;
extern crate rand_xorshift;
use rand_core::SeedableRng;
extern crate getrandom;
extern crate rand;

extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate seed;
use seed::prelude::*;

extern crate wasm_bindgen;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
extern "C" {
    fn prompt() -> Option<String>;
}

#[derive(Clone)]
pub enum Page {
    GenerateSchedule,
    ManagePlayers,
    ManageGroups,
    Preferences,
}

struct Model {
    pub page: Page,
    pub generate_schedule: generate_schedule_page::GenerateSchedule,
    pub manage_players: manage_players_page::ManagePlayers,
    manage_groups: manage_groups_page::ManageGroups,
    preferences: preferences_page::Preferences,
    style_control: style_control::StyleControl,
    database: database::Database,
    rng: rand_xorshift::XorShiftRng,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            page: Page::GenerateSchedule,
            generate_schedule: generate_schedule_page::GenerateSchedule::default(),
            manage_players: manage_players_page::ManagePlayers::default(),
            manage_groups: manage_groups_page::ManageGroups::default(),
            preferences: preferences_page::Preferences::default(),
            style_control: style_control::StyleControl::default(),
            database: database::Database::load(),
            rng: {
                let mut seed: [u8; 16] = [0; 16];
                if getrandom::getrandom(&mut seed).is_err() {
                    alert("Failed to seed RNG");
                };
                rand_xorshift::XorShiftRng::from_seed(seed)
            },
        }
    }
}

#[derive(Clone)]
pub enum Msg {
    ChangePage(Page),
    GSAddPlayer,
    GSAddPlayerSelectBoxInput(String),
    GSAddGroup,
    GSAddGroupSelectBoxInput(String),
    GSRemovePlayer(u32),
    GSRemoveAllPlayers,
    GSSetTables(String),
    GSGenerate,
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
    PSetThemeInput(String),
    PSetTheme,
}

fn update(msg: Msg, model: &mut Model, _: &mut impl Orders<Msg>) {
    match msg {
        Msg::ChangePage(page) => {
            model.page = page;
        }
        Msg::GSAddPlayerSelectBoxInput(id) => {
            model.generate_schedule.set_add_player_select_box_input(id)
        }
        Msg::GSAddPlayer => model.generate_schedule.add_player(&model.database),
        Msg::GSAddGroupSelectBoxInput(id) => {
            model.generate_schedule.set_add_group_select_box_input(id)
        }
        Msg::GSAddGroup => model.generate_schedule.add_group(&model.database),
        Msg::GSRemovePlayer(id) => model.generate_schedule.remove_player(id),
        Msg::GSRemoveAllPlayers => model.generate_schedule.remove_all_players(),
        Msg::GSSetTables(tables) => model.generate_schedule.set_tables(tables),
        Msg::GSGenerate => model.generate_schedule.generate(&mut model.rng),
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
        Msg::PSetThemeInput(theme) => model.preferences.set_theme_input(theme),
        Msg::PSetTheme => model.preferences.set_theme(&mut model.style_control),
    }
}

fn player_select_box(
    database: &database::Database,
    style: &style_control::StyleControl,
) -> Vec<Node<Msg>> {
    let player_list = database.get_players();
    let mut node_list: Vec<Node<Msg>> = Vec::with_capacity(player_list.len() + 1);
    node_list.push(option![style.option_style(), attrs! {At::Value => ""}, ""]);
    for (id, player) in &player_list {
        node_list.push(option![
            style.option_style(),
            attrs! {At::Value => id},
            format!("{} (ID: {})", player.name, id)
        ]);
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
            Page::GenerateSchedule => "Generate Schedule",
            Page::ManagePlayers => "Manage Players",
            Page::ManageGroups => "Manage Groups",
            Page::Preferences => "Preferences",
        }],
        div![
            style![St::Display => "Flex";
		St::FlexWrap => "Wrap";],
            button![
                model.style_control.button_style(),
                &tab_style,
                simple_ev(Ev::Click, Msg::ChangePage(Page::GenerateSchedule)),
                "Generate Schedule"
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
                simple_ev(Ev::Click, Msg::ChangePage(Page::Preferences)),
                "Preferences"
            ]
        ],
        match model.page {
            Page::GenerateSchedule => generate_schedule_page::view_generate_schedule(
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
            Page::Preferences => {
                preferences_page::view_preferences(&model.preferences, &model.style_control)
            }
        },
    ]
}

#[wasm_bindgen(start)]
pub fn render() {
    seed::App::build(|_, _| Init::new(Model::default()), update, view).build_and_start();
}
