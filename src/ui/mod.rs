use std::num::NonZeroU128;

use bevy::{
    diagnostic::RegisterDiagnostic,
    prelude::*,
    render::{
        render_resource::{
            Extent3d, ImageCopyTexture, ImageDataLayout, Origin3d, Texture, TextureAspect,
        },
        renderer::{RenderDevice, RenderQueue},
    },
    window::WindowResized,
};

pub struct AsciiUiPlugin;

impl Plugin for AsciiUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, update_ui_nodes);
    }
}

//=============================================================================
//             Ascii UiComponent
//=============================================================================

#[derive(Default, Component)]
pub struct AsciiUi {
    nodes: Vec<Box<dyn AsciiUiNode + Send + Sync>>,
    is_dirty: bool,
}

impl AsciiUi {
    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    pub fn render(&self, buffer: &mut AsciiBuffer) {
        for node in self.nodes.iter() {
            node.render(buffer);
        }
    }

    pub fn add_node(&mut self, node: impl AsciiUiNode + Send + Sync + 'static) {
        self.is_dirty = true;
        self.nodes.push(Box::new(node));
    }

    pub fn update_nodes(&mut self) {
        self.is_dirty = false;
        let mut nodes = std::mem::take(&mut self.nodes);
        let mut context = AsciiUiContext { ui: self };

        for node in nodes.iter_mut() {
            node.update(&mut context);
        }

        self.nodes = nodes;
    }
}

pub struct AsciiUiContext<'ui> {
    ui: &'ui mut AsciiUi,
}

impl<'ui> AsciiUiContext<'ui> {
    pub fn mark_dirty(&mut self) {
        self.ui.is_dirty = true;
    }
}

//=============================================================================
//             Ascii Buffer
//=============================================================================

pub struct AsciiBuffer {
    data: Vec<AsciiCharacter>,
    width: u32,
    height: u32,
}

impl Default for AsciiBuffer {
    fn default() -> Self {
        AsciiBuffer {
            data: Vec::new(),
            width: 0,
            height: 0,
        }
    }
}

impl AsciiBuffer {
    pub fn from_res(width: u32, height: u32) -> Self {
        let data: Vec<AsciiCharacter> = vec![AsciiCharacter::default(); (width * height) as usize];

        AsciiBuffer {
            data,
            width,
            height,
        }
    }

    pub fn apply(&self, texture: &Texture, queue: &Res<RenderQueue>) {
        queue.write_texture(
            ImageCopyTexture {
                texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            self.as_byte_vec().as_slice(),
            self.image_data_layout(),
            self.size(),
        );
    }

    pub fn as_byte_vec(&self) -> Vec<u8> {
        let result = self
            .data
            .iter()
            .map(|value| value.into_u8())
            .flatten()
            .collect();
        result
    }

    pub fn size(&self) -> Extent3d {
        Extent3d {
            width: self.width,
            height: self.height,
            depth_or_array_layers: 1,
        }
    }

    pub fn image_data_layout(&self) -> ImageDataLayout {
        ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(4 * self.width),
            rows_per_image: Some(self.height),
        }
    }

    pub fn set_character(&mut self, x: u32, y: u32, character: AsciiCharacter) {
        self.data.insert(self.calc_index(x, y), character);
    }

    fn calc_index(&self, x: u32, y: u32) -> usize {
        (x + (y * self.width)) as usize
    }
}

//=============================================================================
//             Ascii Character
//=============================================================================

#[derive(Clone, Copy)]
pub enum AsciiCharacter {
    Set {
        index: u8,
        text_color: u8,
        background_color: u8,
    },
    Unset,
}

impl Default for AsciiCharacter {
    fn default() -> Self {
        AsciiCharacter::Unset
    }
}

impl AsciiCharacter {
    pub const AT_SIGN: AsciiCharacter = AsciiCharacter::Set {
        index: 0,
        text_color: 1,
        background_color: 0,
    };
    pub const A: AsciiCharacter = AsciiCharacter::Set {
        index: 0,
        text_color: 1,
        background_color: 0,
    };
    pub const B: AsciiCharacter = AsciiCharacter::Set {
        index: 1,
        text_color: 1,
        background_color: 0,
    };
    pub const C: AsciiCharacter = AsciiCharacter::Set {
        index: 2,
        text_color: 1,
        background_color: 0,
    };

    pub fn into_u8(&self) -> [u8; 4] {
        match self {
            AsciiCharacter::Set {
                index,
                text_color,
                background_color,
            } => {
                if *index > 127u8 || *text_color > 15u8 || *background_color > 15u8 {
                    return [0, 0, 0, 0];
                } else {
                    return [*index, *text_color, *background_color, 1];
                }
            }
            _ => (),
        }

        [0, 0, 0, 0]
    }
}

//=============================================================================
//             Ascii Ui Node
//=============================================================================

pub trait AsciiUiNode {
    fn render(&self, buffer: &mut AsciiBuffer);

    fn update(&mut self, context: &mut AsciiUiContext);
}

fn update_ui_nodes(
    mut ascii_ui: Query<&mut AsciiUi>,
    mut window_resized: EventReader<WindowResized>,
) {
    for mut ui in ascii_ui.iter_mut() {
        ui.update_nodes();
        if window_resized.len() > 0 {
            ui.is_dirty = true;
        }
    }
}

pub struct TestNode;

impl AsciiUiNode for TestNode {
    fn render(&self, buffer: &mut AsciiBuffer) {
        buffer.set_character(0, 0, AsciiCharacter::A);
    }

    fn update(&mut self, context: &mut AsciiUiContext) {
        // context.mark_dirty();
    }
}
