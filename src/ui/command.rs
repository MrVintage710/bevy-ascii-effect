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
    pub fn top(&mut self, size : impl Into<Value>, component : impl AsciiComponent + Send + Sync + 'static) -> &mut Self {
        self.push_from_pos(AsciiPosition::top(size), component);
        self
    }
    
    pub fn bottom(&mut self, size : impl Into<Value>, component : impl AsciiComponent + Send + Sync + 'static) -> &mut Self {
        self.push_from_pos(AsciiPosition::bottom(size), component);
        self
    }
    
    pub fn left(&mut self, size : impl Into<Value>, component : impl AsciiComponent + Send + Sync + 'static) -> &mut Self {
        self.push_from_pos(AsciiPosition::left(size), component);
        self
    }
    
    pub fn right(&mut self, size : impl Into<Value>, component : impl AsciiComponent + Send + Sync + 'static) -> &mut Self {
        self.push_from_pos(AsciiPosition::right(size), component);
        self
    }
    
    pub fn relative(&mut self, x : i32, y : i32, width : impl Into<Value>, height : impl Into<Value>, component : impl AsciiComponent + Send + Sync + 'static) -> &mut Self {
        self.push_from_pos(AsciiPosition::relavtive(x, y, width, height, self.entity_stack.len() as u32), component);
        self
    }
    
    pub fn centered(&mut self, width : impl Into<Value>, height : impl Into<Value>, component : impl AsciiComponent + Send + Sync + 'static) -> &mut Self {
        self.push_from_pos(AsciiPosition::centered(width, height), component);
        self
    }
    
    pub fn fill(&mut self, component : impl AsciiComponent + Send + Sync + 'static) -> &mut Self {
        self.push_from_pos(AsciiPosition::fill(), component);
        self
    }

    pub fn aligned(&mut self, width : impl Into<Value>, height : impl Into<Value>, ha : HorizontalAlignment, va : VerticalAlignment, component : impl AsciiComponent + Send + Sync + 'static) -> &mut Self {
        self.push_from_pos(AsciiPosition::aligned(width, height, ha, va), component);
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
    
    fn push_from_pos(&mut self, pos : AsciiPosition, component : impl AsciiComponent + Send + Sync + 'static) {
        let parent = self.current_entity.clone();
        let entity = self.commands.spawn((
            pos,
            AsciiNode::default(),
            component,
            VisibilityBundle::default()
        )).id();
        self.commands.entity(parent).add_child(entity.clone());
        self.entity_stack.push_back(parent);
        self.current_entity = entity;
    }
}
