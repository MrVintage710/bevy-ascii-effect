use bevy::{ecs::system::Command, prelude::*};

struct SpawnUiNodesCommand {
    
}

impl Command for SpawnUiNodesCommand {
    fn apply(self, world: &mut World) {
        
    }
}

pub trait AsciiUiCommandExtention {
    fn ascii_ui(&mut self) -> 
}
