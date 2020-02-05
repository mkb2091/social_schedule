use seed::prelude::*;

use crate::{database, style_control, Msg};

pub struct ManageEvents {}

impl Default for ManageEvents {
    fn default() -> Self {
        Self {}
    }
}

pub fn view_manage_events(
    _model: &ManageEvents,
    database: &database::Database,
    style: &style_control::StyleControl,
) -> Node<Msg> {
    let box_style = style![St::PaddingLeft => "15px";
St::PaddingRight => "15px";
St::FlexGrow=> "1";];

    div![
        style![St::Display => "Flex";
        St::FlexWrap => "Wrap"],
        div![
            &box_style,
            table![style![St::PaddingBottom => "5px";], {
                let events_list = database.get_events();
                let mut node_list: Vec<Node<Msg>> = Vec::with_capacity(events_list.len());
                for (&id, event) in &events_list {
                    node_list.push(tr![
                        td![id.to_string()],
                        td![event.name],
                        td![format!("{:} players", event.players.len())],
                        td![button![
                            style.button_style(),
                            raw_ev(Ev::Click, move |_| Msg::MPChangeName(id)),
                            "Change Name"
                        ]],
                    ]);
                }
                node_list
            }],
        ],
    ]
}
