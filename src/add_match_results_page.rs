use seed::prelude::*;

use crate::{
    alert, database, generate_schedule_page, player_select_box, schedule, style_control,
    view_schedule, Msg,
};

#[derive(Default)]
pub struct AddMatchResults {
    expanded_schedules: std::collections::HashSet<u32>,
}

impl AddMatchResults {
    pub fn expand_schedule(&mut self, id: u32) {
        self.expanded_schedules.insert(id);
    }
    pub fn hide_schedule(&mut self, id: u32) {
        self.expanded_schedules.remove(&id);
    }
}

fn view_schedule_with_result_boxes<T: schedule::ScheduleStructure>(
    schedule: &T,
    players: &[u32],
    database: &database::Database,
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
            table.push(tr![{
                let mut row: Vec<Node<Msg>> = Vec::with_capacity(tables + 1);
                row.push(td![format!("Round {:}", round + 1)]);
                for table in 0..tables {
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
                                    input![
                                        attrs! {At::Type => "number"},
                                        style! {St::Width => "3em"; St::FlexGrow => "0"}
                                    ],
                                    br![]
                                ]);
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

pub fn view_add_match_results(
    model: &AddMatchResults,
    database: &database::Database,
    style: &style_control::StyleControl,
) -> Node<Msg> {
    let box_style = style![St::PaddingLeft => "15px";
St::PaddingRight => "15px";
St::FlexGrow=> "1";];

    div![
        h2!["Event List"],
        table![
            style![St::PaddingBottom => "5px";],
            tr![
                td![style![St::PaddingRight => "25px";], "ID"],
                td![style![St::PaddingRight => "25px";], "Name"],
                td![style![St::PaddingRight => "25px";], "Date"],
            ],
            {
                let events_list = database.get_events();
                let mut node_list: Vec<Node<Msg>> = Vec::with_capacity(events_list.len());
                for (&id, event) in &events_list {
                    if event.matches.is_empty() {
                        node_list.push(tr![
                            td![id.to_string()],
                            td![event.name],
                            td![event.date],
                            td![format!("{:} players", event.players.len())],
                            td![if model.expanded_schedules.contains(&id) {
                                button![
                                    style.button_style(),
                                    ev(Ev::Click, move |_| Msg::ARHideSchedule(id)),
                                    "Hide Schedule"
                                ]
                            } else {
                                button![
                                    style.button_style(),
                                    ev(Ev::Click, move |_| Msg::ARExpandSchedule(id)),
                                    "Show Schedule"
                                ]
                            }],
                        ]);
                        node_list.push(tr![td![
                            attrs! {At::ColSpan => 10},
                            div![
                                style![St::Display => "Flex";
        St::FlexWrap => "Wrap"],
                                div![
                                    &box_style,
                                    if model.expanded_schedules.contains(&id) {
                                        view_schedule_with_result_boxes(
                                            &event.schedule,
                                            &event.players,
                                            database,
                                        )
                                    } else {
                                        div![]
                                    }
                                ]
                            ],
                        ],]);
                    }
                }
                node_list
            }
        ],
    ]
}
