use std::collections::VecDeque;

use bevy::{ecs::system::Command, prelude::*};

use super::{buffer::AsciiBounds, node::{AsciiUiComponent, AsciiUiLayout, AsciiUiNode}, AsciiUi};

pub trait AsciiUiCommandExtention<'w, 's> {
    fn ascii_ui<'c>(&'c mut self, parent : Entity) -> AsciiUiCommands<'c, 'w, 's>;
}

impl <'w, 's> AsciiUiCommandExtention<'w, 's> for Commands<'w, 's> {
    fn ascii_ui<'c>(&'c mut self, parent : Entity) -> AsciiUiCommands<'c, 'w, 's> {
        let mut entity_stack = VecDeque::new();
        entity_stack.push_back(parent);
        AsciiUiCommands {
            commands: self,
            entity_stack,
        }
    }
}

pub struct AsciiUiCommands<'c, 'w, 's> {
    commands: &'c mut Commands<'w, 's>,
    entity_stack: VecDeque<Entity>,
}

impl <'c, 'w, 's> AsciiUiCommands<'c, 'w, 's> {
    pub fn absolute(&mut self, bounds : AsciiBounds, component : impl AsciiUiComponent + Send + Sync + 'static) {
        let parent = self.entity_stack.back().unwrap();
        let entity = self.commands.spawn(( 
           Name::new(component.name().to_string()),
           AsciiUiNode {
                layout: AsciiUiLayout::Absolute(bounds.clone()),
                bounds,
                component: Box::new(component),
                hidden: false,
                is_dirty: false,
           }
        )).id();
        self.commands.entity(*parent).add_child(entity.clone());
        self.entity_stack.push_back(entity);
    }
    
    pub fn pop(&mut self) {
        self.entity_stack.pop_back();
    }
}