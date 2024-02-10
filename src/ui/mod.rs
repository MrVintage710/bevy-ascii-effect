pub mod buffer;

use std::{
    default,
    fmt::format,
    num::NonZeroU128,
    rc::Rc,
    sync::{Arc, Mutex},
};

use bevy::{
    diagnostic::RegisterDiagnostic,
    prelude::*,
    render::{
        render_resource::{
            ErasedTexture, Extent3d, ImageCopyTexture, ImageDataLayout, Origin3d, Texture,
            TextureAspect,
        },
        renderer::{RenderDevice, RenderQueue},
    },
    window::{PrimaryWindow, WindowResized},
};
use textwrap::Options;

use crate::ascii::AsciiCamera;

use self::buffer::{AsciiBounds, AsciiBuffer};

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
    nodes: Vec<Arc<Mutex<Box<dyn AsciiUiNode + Send + Sync>>>>,
    is_dirty: bool,
}

impl AsciiUi {
    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    pub fn render(&self, width: u32, height: u32) -> Vec<u8> {
        let buffer = AsciiBuffer::new(width, height);

        for node in self.nodes.iter() {
            node.lock().unwrap().render(&buffer);
        }

        buffer.as_byte_vec()
    }

    pub fn add_node(&mut self, node: impl AsciiUiNode + Send + Sync + 'static) {
        self.is_dirty = true;
        self.nodes.push(Arc::new(Mutex::new(Box::new(node))));
    }

    pub fn update_nodes<'w>(&mut self, cursor_pos : Option<(u32, u32)>, time : &Res<'w, Time>, keys : &Res<'w, Input<KeyCode>>) {
        self.is_dirty = false;
        let mut nodes = std::mem::take(&mut self.nodes);
        
        let mut context = AsciiUiContext { ui: self, cursor_pos, time, key_input : keys};

        for node in nodes.iter_mut() {
            node.lock().unwrap().update(&mut context);
        }

        self.nodes = nodes;
    }
}

pub struct AsciiUiContext<'w, 'ui> {
    ui: &'ui mut AsciiUi,
    cursor_pos: Option<(u32, u32)>,
    time : &'ui Res<'w, Time>,
    key_input : &'ui Res<'w, Input<KeyCode>>,
}

impl<'w, 'ui> AsciiUiContext<'w, 'ui> {
    pub fn mark_dirty(&mut self) {
        self.ui.is_dirty = true;
    }
    
    pub fn cursor_pos(&self) -> Option<(u32, u32)> {
        self.cursor_pos
    }
    
    pub fn time(&self) -> &Time {
        self.time
    }
}

//=============================================================================
//             Styling Constants
//=============================================================================

pub enum BorderType {
    Full,
    Half,
    Dashed,
    None,
}

impl BorderType {
    fn get_character(&self, x: u32, y: u32, width: u32, height: u32) -> Character {
        match self {
            BorderType::Full => {
                if x == 0 && y == 0 {
                    Character::LBorderNW
                } else if x == width - 1 && y == 0 {
                    Character::LBorderNE
                } else if x == 0 && y == height - 1 {
                    Character::LBorderSW
                } else if x == width - 1 && y == height - 1 {
                    Character::LBorderSE
                } else if x == 0 {
                    Character::BorderW
                } else if x == width - 1 {
                    Character::BorderE
                } else if y == 0 {
                    Character::BorderN
                } else if y == height - 1 {
                    Character::BorderS
                } else {
                    Character::Nil
                }
            }
            BorderType::Half => todo!(),
            BorderType::Dashed => todo!(),
            BorderType::None => Character::Nil,
        }
    }
}

#[derive(Default)]
pub enum TextOverflow {
    #[default]
    Hidden,
    Elipses,
}

#[derive(Default)]
pub enum HorizontalAlignment {
    #[default]
    Left,
    Center,
    Right,
}

#[derive(Default)]
pub enum VerticalAlignment {
    #[default]
    Top,
    Center,
    Bottom,
}

pub struct Padding {
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
    pub left: u32,
}

impl Default for Padding {
    fn default() -> Self {
        Padding {
            top: 0,
            right: 0,
            bottom: 0,
            left: 0,
        }
    }
}

impl From<u32> for Padding {
    fn from(padding: u32) -> Self {
        Padding {
            top: padding,
            right: padding,
            bottom: padding,
            left: padding,
        }
    }
}

impl From<(u32, u32)> for Padding {
    fn from((top, right): (u32, u32)) -> Self {
        Padding {
            top,
            right,
            bottom: top,
            left: right,
        }
    }
}

impl From<(u32, u32, u32, u32)> for Padding {
    fn from((top, right, bottom, left): (u32, u32, u32, u32)) -> Self {
        Padding {
            top,
            right,
            bottom,
            left,
        }
    }
}

//=============================================================================
//             Ascii Character
//=============================================================================

#[derive(Clone, Copy)]
pub enum AsciiCharacter {
    Set {
        index: Character,
        text_color: Color,
        background_color: Color,
    },
    Unset,
}

impl Default for AsciiCharacter {
    fn default() -> Self {
        AsciiCharacter::Unset
    }
}

impl AsciiCharacter {
    pub fn new(character: Character, text_color: Color, bg_color: Color) -> AsciiCharacter {
        AsciiCharacter::Set {
            index: character,
            text_color,
            background_color: bg_color,
        }
    }

    pub fn into_u8(&self) -> [u8; 4] {
        match self {
            AsciiCharacter::Set {
                index,
                text_color,
                background_color,
            } => {
                if *index as u8 > 127u8
                    || *text_color as u8 > 15u8
                    || *background_color as u8 > 15u8
                {
                    return [0, 0, 0, 0];
                } else {
                    return [*index as u8, *text_color as u8, *background_color as u8, 1];
                }
            }
            _ => (),
        }

        [0, 0, 0, 0]
    }
}

impl From<Character> for AsciiCharacter {
    fn from(value: Character) -> Self {
        AsciiCharacter::new(value, Color::White, Color::Black)
    }
}

impl From<(Character, Color)> for AsciiCharacter {
    fn from(value: (Character, Color)) -> Self {
        AsciiCharacter::new(value.0, value.1, Color::Black)
    }
}

impl From<(Character, Color, Color)> for AsciiCharacter {
    fn from(value: (Character, Color, Color)) -> Self {
        AsciiCharacter::new(value.0, value.1, value.2)
    }
}

impl From<Color> for AsciiCharacter {
    fn from(value: Color) -> Self {
        AsciiCharacter::new(Character::Dither, value, value)
    }
}

impl From<char> for AsciiCharacter {
    fn from(value: char) -> Self {
        AsciiCharacter::new(value.into(), Color::White, Color::Black)
    }
}

impl From<(char, Color)> for AsciiCharacter {
    fn from(value: (char, Color)) -> Self {
        AsciiCharacter::new(value.0.into(), value.1, Color::Black)
    }
}

impl From<(char, Color, Color)> for AsciiCharacter {
    fn from(value: (char, Color, Color)) -> Self {
        AsciiCharacter::new(value.0.into(), value.1, value.2)
    }
}

//=============================================================================
//             Ascii Ui Node
//=============================================================================

pub trait AsciiUiNode {
    fn render(&mut self, buffer: &AsciiBuffer);

    fn update(&mut self, context: &mut AsciiUiContext);
}

fn update_ui_nodes(
    mut ascii_ui: Query<(&mut AsciiUi, &AsciiCamera)>,
    window_resized: EventReader<WindowResized>,
    time : Res<Time>,
    window : Query<&Window, With<PrimaryWindow>>,
    key_input : Res<Input<KeyCode>>,
) {
    for (mut ui, camera) in ascii_ui.iter_mut() {
        let window = window.single();
        let cursor_pos = window.cursor_position();
        let target_res = camera.target_res();
        let target_cursor_pos = if let Some(cursor_pos) = cursor_pos {
            let pixel_multiplier = (window.physical_width() as f32 / target_res.x) / 2.0;
            let x = (cursor_pos.x / pixel_multiplier).floor() as u32;
            let y = (cursor_pos.y / pixel_multiplier).floor() as u32;
            Some((x, y))
        } else {
            None
        };
        ui.update_nodes(target_cursor_pos, &time, &key_input);
        if window_resized.len() > 0 {
            ui.is_dirty = true;
        }
    }
    
}

pub struct TestNode {
    dims : AsciiBounds,
    color : Color
}

impl Default for TestNode {
    fn default() -> Self {
        TestNode {
            dims : AsciiBounds::from_dims(40, 20),
            color : Color::Violet
        }
    }
}

impl AsciiUiNode for TestNode {
    fn render(&mut self, buffer: &AsciiBuffer) {
        let center = buffer.center(self.dims.width, self.dims.height);
        
        self.dims = center.clone().bounds;
        let inner_square = center
            .square()
            .border(BorderType::Full)
            .title("Centered Box")
            .border_color(self.color)
            .draw();
        
        if let Some(inner_square) = inner_square {
            if let Some(splits) = inner_square.vertical_split::<2>() {
                splits[0].padding((0, 1, 0, 0)).text("This text should be on the left, and it should wrap to the next line.").wrap().draw();
                splits[1].text("This text should be on the right, and it should wrap to the next line.").wrap().draw();
            }
        }
    }

    fn update(&mut self, context: &mut AsciiUiContext) {
        let Some(cursor) = context.cursor_pos() else {return;};
        if self.dims.is_within(cursor.0, cursor.1) {
            if self.color != Color::Red {context.mark_dirty()}
            self.color = Color::Red;
        } else {
            if self.color != Color::Violet {context.mark_dirty()}
            self.color = Color::Violet;
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Character {
    AT,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    LeftBracket,
    Euro,
    RightBracket,
    ArrowUp,
    ArrowLeft,
    Space,
    ExcalamationMark,
    DoubleQuotes,
    Hashtag,
    Dollar,
    Percent,
    Ampersand,
    Apostrophe,
    LeftParenthesis,
    RightParenthesis,
    Asterisk,
    Plus,
    Comma,
    Hyphen,
    Period,
    ForwardSlash,
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Colon,
    SemiColon,
    LessThan,
    Equal,
    GreaterThan,
    QuestionMark,
    DashedHorizontalCenter,
    Spade,
    BorderVerticalCenter,
    BorderHorizontalCenter,
    BorderHorizontalN2,
    BorderHorizontalN4,
    BorderHorizontalS2,
    BorderVerticalW2,
    BorderVerticalE2,
    RoundedCornerCenterNE,
    RoundedCornerCenterSW,
    RoundedCornerCenterSE,
    LBorderSW,
    DiagonalEB,
    DiagonalWB,
    LBorderNW,
    LBorderNE,
    Circle,
    BorderHorizontalS4,
    Heart,
    BorderVerticalW4,
    RoundedCornerNW,
    DiagonalCross,
    Doughnut,
    Sign,
    BorderVerticalE4,
    Ball,
    Cross,
    DitherW,
    DashedVerticalCenter,
    Pi,
    StairNE,
    Nil,
    HalfW,
    HalfS,
    ThinBorderN,
    ThinBorderS,
    BorderW,
    Dither,
    BorderE,
    DitherS,
    StairsNW,
    DashedE,
    TBorderNSE,
    QuadSE,
    CornerNE,
    CornerWS,
    BorderS,
    CornerNW,
    TBorderNWE,
    TBorderSWE,
    TBorderNSW,
    DashedW,
    ThickBorderW,
    ThickBorderE,
    BorderN,
    ThickBorderN,
    ThickBorderS,
    LBorderSE,
    QuadSW,
    QuadNE,
    CornerSE,
    QuadNW,
    QuadCorners,
}

impl From<char> for Character {
    fn from(value: char) -> Self {
        match value {
            '@' => Character::AT,
            'a' | 'A' => Character::A,
            'b' | 'B' => Character::B,
            'c' | 'C' => Character::C,
            'd' | 'D' => Character::D,
            'e' | 'E' => Character::E,
            'f' | 'F' => Character::F,
            'g' | 'G' => Character::G,
            'h' | 'H' => Character::H,
            'i' | 'I' => Character::I,
            'j' | 'J' => Character::J,
            'k' | 'K' => Character::K,
            'l' | 'L' => Character::L,
            'm' | 'M' => Character::M,
            'n' | 'N' => Character::N,
            'o' | 'O' => Character::O,
            'p' | 'P' => Character::P,
            'q' | 'Q' => Character::Q,
            'r' | 'R' => Character::R,
            's' | 'S' => Character::S,
            't' | 'T' => Character::T,
            'u' | 'U' => Character::U,
            'v' | 'V' => Character::V,
            'w' | 'W' => Character::W,
            'x' | 'X' => Character::X,
            'y' | 'Y' => Character::Y,
            'z' | 'Z' => Character::Z,
            '[' => Character::LeftBracket,
            '\u{20AC}' => Character::Euro,
            ']' => Character::RightBracket,
            '\u{2191}' => Character::ArrowUp,
            '\u{2190}' => Character::ArrowLeft,
            ' ' => Character::Space,
            '!' => Character::ExcalamationMark,
            '"' => Character::DoubleQuotes,
            '#' => Character::Hashtag,
            '$' => Character::Dollar,
            '%' => Character::Percent,
            '&' => Character::Ampersand,
            '\'' => Character::Apostrophe,
            '(' => Character::LeftParenthesis,
            ')' => Character::RightParenthesis,
            '*' => Character::Asterisk,
            '+' => Character::Plus,
            ',' => Character::Comma,
            '-' => Character::Hyphen,
            '.' => Character::Period,
            '/' => Character::ForwardSlash,
            '0' => Character::Zero,
            '1' => Character::One,
            '2' => Character::Two,
            '3' => Character::Three,
            '4' => Character::Four,
            '5' => Character::Five,
            '6' => Character::Six,
            '7' => Character::Seven,
            '8' => Character::Eight,
            '9' => Character::Nine,
            ':' => Character::Colon,
            ';' => Character::SemiColon,
            '<' => Character::LessThan,
            '=' => Character::Equal,
            '>' => Character::GreaterThan,
            '?' => Character::QuestionMark,
            _ => Character::Nil,
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Color {
    Black,
    White,
    Red,
    Cyan,
    Violet,
    Green,
    Blue,
    Yellow,
    Orange,
    Brown,
    LightRed,
    DarkGrey,
    Grey,
    LightGreen,
    LightBlue,
    LightGrey,
}
