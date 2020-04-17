use seed::prelude::*;

#[derive(Clone, Copy)]
pub enum Theme {
    Dark,
    Light,
}

pub struct StyleControl {
    theme: Theme,
}

impl Default for StyleControl {
    fn default() -> Self {
        Self {
            theme: Theme::Light,
        }
    }
}

impl StyleControl {
    pub fn base_style(&self) -> seed::virtual_dom::style::Style {
        match self.theme {
            Theme::Light => style![
            St::Background => "#FFFFFF";
            St::Color => "#000000";
                        ],
            Theme::Dark => style![
St::Background => "#171717";
St::Color => "#FFFFFF";],
        }
    }

    pub fn button_style(&self) -> seed::virtual_dom::style::Style {
        match self.theme {
            Theme::Light => style![St::Border => "1px solid #CCCCCC";
            St::BorderRadius => "3px";
            St::Padding => "5px 10px 5px 10px";
            St::BackgroundImage => "linear-gradient(to bottom, #F7F5F6, #DDDDDD)";
            ],
            Theme::Dark => style![St::Border => "1px solid #202020";
            St::BorderRadius => "3px";
            St::Padding => "5px 10px 5px 10px";
            St::BackgroundImage => "linear-gradient(to bottom, #242424, #101010)";
            St::Color => "#FFFFFF";
            ],
        }
    }
    pub fn option_style(&self) -> seed::virtual_dom::style::Style {
        match self.theme {
            Theme::Light => style![St::Background => "#FFFFFF";
            St::Color => "#000000";
                        ],
            Theme::Dark => style![St::Background => "#171717";
            St::Color => "#FFFFFF";
            ],
        }
    }
    pub fn input_style(&self) -> seed::virtual_dom::style::Style {
        match self.theme {
            Theme::Light => style![St::Border => "1px solid #CCCCCC";
            St::BorderRadius => "3px";
            St::Padding => "5px 10px 5px 10px";
            ],
            Theme::Dark => style![St::Border => "1px solid #202020";
            St::BorderRadius => "3px";
            St::Padding => "5px 10px 5px 10px";
            St::Color => "#FFFFFF";
            ],
        }
    }
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }
}
