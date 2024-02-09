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

use self::buffer::AsciiBuffer;

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

    pub fn render(&self, width: u32, height: u32) -> Vec<u8> {
        let mut buffer = AsciiBuffer::new(width, height);

        for node in self.nodes.iter() {
            node.render(&mut buffer);
        }

        buffer.as_byte_vec()
    }

    pub fn add_node(&mut self, node: impl AsciiUiNode + Send + Sync + 'static) {
        self.is_dirty = true;
        self.nodes.push(Box::new(node));
    }

    pub fn update_nodes<'w>(&mut self, cursor_pos : Option<(u32, u32)>, time : &Res<'w, Time>, keys : &Res<'w, Input<KeyCode>>) {
        self.is_dirty = false;
        let mut nodes = std::mem::take(&mut self.nodes);
        
        let mut context = AsciiUiContext { ui: self, cursor_pos, time, key_input : keys};

        for node in nodes.iter_mut() {
            node.update(&mut context);
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
    Wrap,
    Elipses,
    None,
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
    fn render(&self, buffer: &mut AsciiBuffer);

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
            let x = (cursor_pos.x / target_res.x).floor() as u32;
            let y = (cursor_pos.y / target_res.y).floor() as u32;
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
    width : u32,
    height : u32
}

impl Default for TestNode {
    fn default() -> Self {
        TestNode {
            width : 40,
            height : 20
        }
    }
}

impl AsciiUiNode for TestNode {
    fn render(&self, buffer: &mut AsciiBuffer) {
        let center = buffer.center(self.width, self.height);
        
        center
            .square()
            .border(BorderType::Full)
            .title("Centered Box")
            .border_color(Color::Violet)
            .draw();
    }

    fn update(&mut self, context: &mut AsciiUiContext) {
        println!("{:?}", context.time().delta());
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
