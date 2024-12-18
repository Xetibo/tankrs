use bevy_iced::iced::{widget::container::Style, Background, Color, Theme};

pub struct BlackBackgroundContainer;

pub fn something(theme: &Theme) -> Style {
    let palette = theme.extended_palette();
    Style {
        text_color: Some(palette.primary.base.text),
        background: Some(Background::Color(Color::BLACK)),
        ..Default::default()
    }
}

pub fn get_custom_container_style(theme: &Theme) -> Style {
    something(theme)
}
