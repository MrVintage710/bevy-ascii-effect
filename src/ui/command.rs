use std::collections::VecDeque;

use bevy::prelude::*;

use super::{bounds::{AsciiBounds, AsciiNode}, component::AsciiComponent, position::AsciiPosition, util::Value, HorizontalAlignment, VerticalAlignment};

pub trait AsciiUiCommandExtention<'w, 's> {
    fn ascii_ui_with_parent<'c>(&'c mut self, parent: Entity) -> AsciiUiCommands<'c, 'w, 's>;
    
    fn ascii_ui<'c>(&'c mut self) -> AsciiUiCommands<'c, 'w, 's>;
}

impl<'w, 's> AsciiUiCommandExtention<'w, 's> for Commands<'w, 's> {
    fn ascii_ui_with_parent<'c>(&'c mut self, parent: Entity) -> AsciiUiCommands<'c, 'w, 's> {
        AsciiUiCommands {
            commands: self,
            entity_stack : VecDeque::new(),
            current_entity: parent,
        }
    }
    
    fn ascii_ui<'c>(&'c mut self) -> AsciiUiCommands<'c, 'w, 's> {
        let parent = self.spawn_empty().id();
        AsciiUiCommands {
            commands: self,
            entity_stack : VecDeque::new(),
            current_entity: parent,
        }
    }
}

pub struct AsciiUiCommands<'c, 'w, 's> {
    commands: &'c mut Commands<'w, 's>,
    entity_stack: VecDeque<Entity>,
    current_entity: Entity,
}

impl<'c, 'w, 's> AsciiUiCommands<'c, 'w, 's> {
    pub fn relative(&mut self, x : i32, y : i32, width : impl Into<Value>, height : impl Into<Value>, component : impl AsciiComponent + Send + Sync + 'static) -> &mut Self {
        let parent = self.current_entity.clone();
        let entity = self.commands.spawn((
           AsciiPosition::Relative { x, y, width : width.into(), height : height.into(), layer : self.entity_stack.len() as u32 },
           AsciiNode::default(),
           component,
           VisibilityBundle::default()
        )).id();
        self.commands.entity(parent).add_child(entity.clone());
        self.entity_stack.push_back(entity);
        self.current_entity = entity;
        self
    }

    pub fn aligned(&mut self, width : impl Into<Value>, height : impl Into<Value>, ha : HorizontalAlignment, va : VerticalAlignment, component : impl AsciiComponent + Send + Sync + 'static) -> &mut Self {
        let parent = self.current_entity.clone();
        let entity = self.commands.spawn((
           AsciiPosition::Aligned { width : width.into(), height : height.into(), horizontal: ha, vertical: va },
           AsciiNode::default(),
           component,
           VisibilityBundle::default()
        )).id();
        self.commands.entity(parent).add_child(entity.clone());
        self.entity_stack.push_back(entity);
        self.current_entity = entity;
        self
    }
    
    pub fn child(&mut self, component : impl AsciiComponent + Send + Sync + 'static) -> &mut Self {
        let parent = self.current_entity.clone();
        let entity = self.commands.spawn((
           AsciiPosition::Relative { x: 0, y: 0, width : Value::Percent(1.0), height : Value::Percent(1.0), layer : self.entity_stack.len() as u32 },
           AsciiNode::default(),
           component,
           VisibilityBundle::default()
        )).id();
        self.commands.entity(parent).add_child(entity.clone());
        self.entity_stack.push_back(entity);
        self.current_entity = entity;
        self
    }
    
    pub fn hidden(&mut self) -> &mut Self {
        self.commands.entity(self.current_entity).insert(Visibility::Hidden);
        self
    }
    
    pub fn named(&mut self, name : &str) -> &mut Self {
        self.commands.entity(self.current_entity).insert(Name::new(name.to_string()));
        self
    }
    
    pub fn insert(&mut self, bundle : impl Bundle) -> &mut Self {
        self.commands.entity(self.current_entity).insert(bundle);
        self
    }

    pub fn pop(&mut self) -> &mut Self {
        if !self.entity_stack.is_empty() {
            self.current_entity = self.entity_stack.pop_back().unwrap();
        }
        self
    }
}
