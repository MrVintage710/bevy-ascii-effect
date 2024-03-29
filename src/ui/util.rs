
use std::ops::{Deref, DerefMut};

use bevy::{prelude::*, render::camera::RenderTarget, window::PrimaryWindow};

use crate::ascii::AsciiCamera;

//=============================================================================
//            UtilPlugin
//=============================================================================

pub struct AsciiUtils;

impl Plugin for AsciiUtils {
    fn build(&self, app: &mut App) {
        app
            .register_type::<AsciiCursor>()
            .add_event::<AsciiComponentHoverEnteredEvent>()
            .add_event::<AsciiComponentHoverExitedEvent>()
            .add_event::<AsciiComponentButtonClicked>()
            .add_systems(PreUpdate, update_ascii_cursor)
        ;
    }
}

//=============================================================================
//            AsciiCursor
//=============================================================================

#[derive(Component, Reflect, Clone, Debug, PartialEq, Eq, Hash)]
pub enum AsciiCursor {
    Some {
        x: u32,
        y: u32,
    },
    None
}

fn update_ascii_cursor(
    mut commands : Commands,
    mut primary_window : Query<(Entity, &Window, Option<&mut AsciiCursor>), With<PrimaryWindow>>,
    mut windows : Query<(&Window, Option<&mut AsciiCursor>), Without<PrimaryWindow>>,
    cameras : Query<(&Camera, &AsciiCamera)>,
) {
    for (camera, ascii) in cameras.iter() {
        if let RenderTarget::Window(window_ref ) = camera.target {
            let (window_entity, window, cursor) = match window_ref {
                bevy::window::WindowRef::Primary => {
                    let Ok(window) = primary_window.get_single_mut() else {
                        return;
                    };
                    window
                },
                bevy::window::WindowRef::Entity(entity) => {
                    let Ok((window, cursor)) = windows.get_mut(entity) else {
                        return;
                    };
                    (entity, window, cursor)
                },
            };
            
            match (window.cursor_position(), cursor) {
                (None, None) => {commands.entity(window_entity).insert(AsciiCursor::None);},
                (None, Some(mut cursor)) => *cursor = AsciiCursor::None,
                (Some(pos), None) => {
                    let target_res = ascii.target_res();
                    let x = (pos.x / (window.physical_width() as f32 /  target_res.x)).floor() as u32;
                    let y = (pos.y / (window.physical_height() as f32 /  target_res.y)).floor() as u32;
                    commands.entity(window_entity).insert(AsciiCursor::Some { x, y });
                },
                (Some(pos), Some(mut cursor)) => {
                    let target_res = ascii.target_res();
                    let x = ((pos.x * 2.0) / (window.physical_width() as f32 /  target_res.x)).floor() as u32;
                    let y = ((pos.y * 2.0) / (window.physical_height() as f32 /  target_res.y)).floor() as u32 ;
                    *cursor = AsciiCursor::Some { x, y };
                },
            };
        }
    }
}

//=============================================================================
//            Ui Events
//=============================================================================

#[derive(Event, Reflect, Clone, Debug, PartialEq, Eq, Hash)]
pub struct AsciiComponentHoverEnteredEvent(pub Entity);

#[derive(Event, Reflect, Clone, Debug, PartialEq, Eq, Hash)]
pub struct AsciiComponentHoverExitedEvent(pub Entity);

#[derive(Event, Reflect, Clone, Debug, PartialEq, Eq, Hash)]
pub struct AsciiComponentButtonClicked(pub Entity);

//=============================================================================
//            Ui Events
//=============================================================================

#[derive(Debug, Clone, PartialEq, Copy, Reflect)]
pub enum Value {
    Px(i32),
    Percent(f32),
}

impl Value {
    pub fn pixel_u32(&self, parent_dim : u32) -> u32 {
        match self {
            Value::Px(v) => *v as u32,
            Value::Percent(v) => (parent_dim as f32 * v) as u32,
        }
    }
}

impl From<f32> for Value {
    fn from(v: f32) -> Self {
        Value::Percent(v)
    }
}

impl From<i32> for Value {
    fn from(v: i32) -> Self {
        Value::Px(v)
    }
}

//=============================================================================
//            Ui Value Change Detection
//=============================================================================

#[derive(Reflect, Debug, Clone, Copy)]
pub struct Variable<T> {
    value : T,
    changed : bool,
}

impl <T> Variable<T> {
    pub fn new(value : T) -> Self {
        Self {
            value,
            changed : false,
        }
    }

    pub fn changed(&self) -> bool {
        self.changed
    }
    
    pub fn reset(&mut self) {
        self.changed = false;
    }
}

impl <T> Default for Variable<T> where T : Default {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl <T> From<T> for Variable<T> {
    fn from(value : T) -> Self {
        Self::new(value)
    }
}

impl <T> Deref for Variable<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl <T> DerefMut for Variable<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.changed = true;
        &mut self.value
    }
}

//=============================================================================
//            Line Braker
//=============================================================================

pub fn break_string_into_lines(string : &str, max_width : usize) -> Vec<String> {
    textwrap::wrap(string, max_width).iter().map(|s| s.to_string()).collect()
}