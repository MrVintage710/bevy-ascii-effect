use bevy::{
    ecs::system::SystemParam, prelude::*, window::PrimaryWindow
};

use crate::{prelude::{HorizontalAlignment, VerticalAlignment}, ui::{buffer::AsciiBuffer, character::Color, util::{AsciiComponentButtonClicked, AsciiComponentHoverEnteredEvent, AsciiComponentHoverExitedEvent, AsciiCursor}, AsciiMarkDirtyEvent, BorderType}};

use super::{super::bounds::AsciiBounds, AsciiComponent};

#[derive(Component, Reflect)]
pub struct AsciiButton {
    bg_color: Color,
    border_color: Color,
    text_color: Color,
    hover_bg_color: Color,
    hover_border_color: Color,
    hover_text_color: Color,
    is_hovering: bool,
    button_text: String,
}

impl AsciiButton {
    pub fn from_string(text: &str) -> Self {
        AsciiButton {
            bg_color: Color::Black,
            border_color: Color::White,
            text_color: Color::White,
            hover_bg_color: Color::Grey,
            hover_border_color: Color::White,
            hover_text_color: Color::White,
            is_hovering: false,
            button_text: text.to_string(),
        }
    }
}

impl AsciiComponent for AsciiButton {
    type UpdateQuery<'w, 's> = (
        Query<'w, 's, &'static AsciiCursor, With<PrimaryWindow>>,
        Res<'w, ButtonInput<MouseButton>>,
        EventWriter<'w, AsciiComponentHoverEnteredEvent>,
        EventWriter<'w, AsciiComponentHoverExitedEvent>,
        EventWriter<'w, AsciiComponentButtonClicked>,
        EventWriter<'w, AsciiMarkDirtyEvent>
    );

    fn render(&self, buffer: &mut AsciiBuffer) {
        if let Some(inner) = buffer
            .square()
            .border(BorderType::Full)
            .bg_color(if self.is_hovering { self.hover_bg_color } else { self.bg_color })
            .border_color(if self.is_hovering { self.hover_border_color } else { self.border_color })
            .draw()
        {
            inner
                .text(&self.button_text)
                .vertical_alignment(VerticalAlignment::Center)
                .horizontal_alignment(HorizontalAlignment::Center)
                .text_color(if self.is_hovering { self.hover_text_color } else { self.text_color })
                .bg_color(if self.is_hovering { self.hover_bg_color } else { self.bg_color })
                .draw();
        }
    }

    fn update(
        &mut self,
        query: &mut <Self::UpdateQuery<'_, '_> as SystemParam>::Item<'_, '_>,
        bounds: &AsciiBounds,
        entity : Entity
    ) {
        let Ok(cursor) = query.0.get_single() else {return};
        
        if let AsciiCursor::Some { x, y } = cursor {
            if bounds.is_within(*x as i32, *y as i32) {
                if !self.is_hovering {
                    query.2.send(AsciiComponentHoverEnteredEvent(entity));
                    query.5.send(AsciiMarkDirtyEvent);
                }
                self.is_hovering = true;
            } else {
                if self.is_hovering {
                    query.3.send(AsciiComponentHoverExitedEvent(entity));
                    query.5.send(AsciiMarkDirtyEvent);
                }
                self.is_hovering = false;
            }
        }
        
        if query.1.just_pressed(MouseButton::Left) && self.is_hovering {
            query.4.send(AsciiComponentButtonClicked(entity));
        }
    }
}
