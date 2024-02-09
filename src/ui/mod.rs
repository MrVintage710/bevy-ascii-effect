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
    window::WindowResized,
};
use textwrap::Options;

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
        let mut data: Vec<AsciiCharacter> =
            vec![AsciiCharacter::default(); (width * height) as usize];

        let mut buffer = AsciiBuffer {
            surface: Rc::new(Mutex::new(data)),
            surface_width: width,
            surface_height: height,
            width,
            height,
            x: 0,
            y: 0,
        };

        for node in self.nodes.iter() {
            node.render(&mut buffer);
        }

        buffer.as_byte_vec()
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
    pub surface: Rc<Mutex<Vec<AsciiCharacter>>>,
    surface_width: u32,
    surface_height: u32,
    width: u32,
    height: u32,
    x: u32,
    y: u32,
}

impl AsciiBuffer {
    pub fn set_character(&self, x: u32, y: u32, character: impl Into<AsciiCharacter>) {
        if self.is_within(x, y) {
            let index = self.calc_index(x, y);
            if ((self.surface_width * self.surface_height) as usize) > index {
                let mut surface = self.surface.lock().expect(
                    "There has been an error writing to the Ascii Overlay. Mutex is Poisoned.",
                );
                surface[index] = character.into();
            }
        }
    }

    pub fn sub_buffer(&self, x: u32, y: u32, width: u32, height: u32) -> Option<AsciiBuffer> {
        if self.is_within(x, y) {
            let width = self.width.saturating_sub(x).min(width);
            let height = self.height.saturating_sub(y).min(height);

            return Some(AsciiBuffer {
                surface: self.surface.clone(),
                surface_width: self.surface_width,
                surface_height: self.surface_height,
                width,
                height,
                x,
                y,
            });
        }

        None
    }

    pub fn center(&self, width: u32, height: u32) -> AsciiBuffer {
        AsciiBuffer {
            surface: self.surface.clone(),
            surface_width: self.surface_width,
            surface_height: self.surface_width,
            width : width.min(self.width),
            height : height.min(self.height),
            x: ((self.width / 2) - (width / 2)).max(0),
            y: ((self.height / 2) - (height / 2)).max(0),
        }
    }

    pub fn square(&self) -> AsciiBoxDrawer {
        AsciiBoxDrawer {
            buffer: self,
            bg_color: Color::Black,
            border_color: Color::White,
            title_color: Color::Black,
            title_bg_color: None,
            title: None,
            title_alignment: HorizontalAlignment::Left,
            title_overflow: TextOverflow::default(),
            border: BorderType::None,
        }
    }

    pub fn text(&mut self, x: u32, y: u32, text: &str) {
        for (i, c) in text.chars().enumerate() {
            self.set_character(x + i as u32, y, c)
        }
    }

    pub fn text_color(&mut self, x: u32, y: u32, text: &str, text_color: Color, bg_color: Color) {
        for (i, c) in text.chars().enumerate() {
            self.set_character(x + i as u32, y, (c, text_color, bg_color))
        }
    }

    fn calc_index(&self, x: u32, y: u32) -> usize {
        let x = self.x + x;
        let y = self.y + y;
        (x + (y * self.surface_width)) as usize
    }

    fn is_within(&self, x: u32, y: u32) -> bool {
        let x = self.x + x;
        let y = self.y + y;
        x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
    }

    fn as_byte_vec(&self) -> Vec<u8> {
        let result = self
            .surface
            .lock()
            .expect("Error while rendering Ascii overlay.")
            .iter()
            .map(|value| value.into_u8())
            .flatten()
            .collect();
        result
    }
}

//=============================================================================
//             Ascii Box Drawer
//=============================================================================

pub struct AsciiBoxDrawer<'b> {
    buffer: &'b AsciiBuffer,
    bg_color: Color,
    border_color: Color,
    title_color: Color,
    title_bg_color: Option<Color>,
    title: Option<String>,
    title_alignment: HorizontalAlignment,
    title_overflow: TextOverflow,
    border: BorderType,
}

impl<'b> AsciiBoxDrawer<'b> {
    pub fn draw(mut self) -> Option<AsciiBuffer> {
        for y in 0..self.buffer.height {
            for x in 0..self.buffer.width {
                let character = self.calc_character(x, y);
                self.buffer.set_character(x, y, character);
            }
        }

        self.buffer
            .sub_buffer(1, 1, self.buffer.width - 2, self.buffer.height - 2)
    }

    fn calc_character(&mut self, x: u32, y: u32) -> AsciiCharacter {
        let max_title_width = self.buffer.width as i32 - 4;
        let character = self
            .border
            .get_character(x, y, self.buffer.width, self.buffer.height);
        if max_title_width < 2 {
            return (character, self.border_color, self.bg_color).into();
        }

        if let Some(title) = &self.title {
            if y == 0 && x >= 2 && x <= self.buffer.width - 2 {
                let title_len = title.len().min(max_title_width as usize);
                // let difference = title_len as i32 - max_title_width;
                let x_start = match self.title_alignment {
                    HorizontalAlignment::Left => 2,
                    HorizontalAlignment::Center => {
                        (self.buffer.width / 2 - title_len as u32 / 2).max(2)
                    }
                    HorizontalAlignment::Right => {
                        (self.buffer.width as i32 - title_len as i32 - 2).max(2) as u32
                    }
                };

                let index = x as i32 - x_start as i32;

                if index >= 0 && index < title_len as i32 {
                    let c: Character = title.chars().nth(index as usize).unwrap().into();
                    return (
                        c,
                        self.title_color,
                        self.title_bg_color.unwrap_or(self.border_color),
                    )
                        .into();
                }
            }
        }

        (character, self.border_color, self.bg_color).into()
    }

    pub fn title_alignment(mut self, alignment: HorizontalAlignment) -> Self {
        self.title_alignment = alignment;
        self
    }

    pub fn title_overflow(mut self, overflow: TextOverflow) -> Self {
        self.title_overflow = overflow;
        self
    }

    pub fn bg_color(mut self, bg_color: Color) -> Self {
        self.bg_color = bg_color;
        self
    }

    pub fn border_color(mut self, border_color: Color) -> Self {
        self.border_color = border_color;
        self
    }

    pub fn border(mut self, border_type: BorderType) -> Self {
        self.border = border_type;
        self
    }

    pub fn title_text_color(mut self, text_color: Color) -> Self {
        self.title_color = text_color;
        self
    }

    pub fn title_bg_color(mut self, bg_color: Color) -> Self {
        self.title_bg_color = Some(bg_color);
        self
    }

    pub fn title(mut self, title: &str) -> Self {
        self.title = Some(title.to_string());
        self
    }
}

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
        buffer
            .center(40, 20)
            .square()
            .border(BorderType::Full)
            .title("Centered Box")
            .draw();
    }

    fn update(&mut self, context: &mut AsciiUiContext) {
        // context.mark_dirty();
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
