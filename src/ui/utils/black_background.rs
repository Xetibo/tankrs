use bevy_iced::iced::{widget::container, Background, Color};

pub struct BlackBackgroundContainer;

impl container::StyleSheet for BlackBackgroundContainer {
    type Style = bevy_iced::iced::Theme;

    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        let palette = style.palette();
        container::Appearance {
            text_color: Some(palette.text),
            background: Some(Background::Color(Color::BLACK)),
            ..Default::default()
        }
    }
}

pub fn get_custom_container_style() -> bevy_iced::iced::theme::Container {
    bevy_iced::iced::theme::Container::Custom(Box::new(BlackBackgroundContainer))
}
