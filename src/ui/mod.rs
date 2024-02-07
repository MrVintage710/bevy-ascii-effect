use std::{
    num::NonZeroU128,
    sync::{Arc, Mutex},
};

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
            surface: Mutex::new(data),
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
    pub surface: Mutex<Vec<AsciiCharacter>>,
    surface_width: u32,
    surface_height: u32,
    width: u32,
    height: u32,
    x: u32,
    y: u32,
}

impl AsciiBuffer {
    pub fn set_character(&mut self, x: u32, y: u32, character: impl Into<AsciiCharacter>) {
        if self.is_within(x, y) {
            let index = self.calc_index(x, y);
            if (self.surface_width * self.surface_height) as usize > index {
                let mut surface = self.surface.get_mut().expect(
                    "There has been an error writing to the Ascii Overlay. Mutex is Poisoned.",
                );
                surface[index] = character.into();
            }
        }
    }

    pub fn sub_buffer(&self, x: u32, y: u32, width: u32, height: u32) -> AsciiBuffer {
        if self.is_within(x, y) {}
    }

    pub fn filled_border_box(
        &mut self,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        border_color: Color,
        fill_color: Color,
    ) {
        for h in 0..height {
            for w in 0..width {
                let pos_x = w + x;
                let pos_y = h + y;
                if w == 0 && h == 0 {
                    self.set_character(
                        pos_x,
                        pos_y,
                        (Character::LBorderNW, border_color, fill_color),
                    );
                } else if w == (width - 1) && h == 0 {
                    self.set_character(
                        pos_x,
                        pos_y,
                        (Character::LBorderNE, border_color, fill_color),
                    )
                } else if w == 0 && h == (height - 1) {
                    self.set_character(
                        pos_x,
                        pos_y,
                        (Character::LBorderSW, border_color, fill_color),
                    )
                } else if w == (width - 1) && h == (height - 1) {
                    self.set_character(
                        pos_x,
                        pos_y,
                        (Character::LBorderSE, border_color, fill_color),
                    )
                } else if w == 0 {
                    self.set_character(pos_x, pos_y, (Character::BorderW, border_color, fill_color))
                } else if w == (width - 1) {
                    self.set_character(pos_x, pos_y, (Character::BorderE, border_color, fill_color))
                } else if h == 0 {
                    self.set_character(pos_x, pos_y, (Character::BorderN, border_color, fill_color))
                } else if h == (height - 1) {
                    self.set_character(pos_x, pos_y, (Character::BorderS, border_color, fill_color))
                } else {
                    self.set_character(pos_x, pos_y, fill_color)
                }
            }
        }
    }

    pub fn text_box_color(
        &mut self,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        text: &str,
        text_color: Color,
        bg_color: Color,
    ) {
        let text = textwrap::wrap(text, Options::new(width as usize));

        for row in 0..height {
            if text.len() <= row as usize {
                break;
            }

            let line = &text[row as usize];

            if row != (height - 1) {
                self.text_color(x, y + row, &line, text_color, bg_color);
            } else {
                let text = line.split_at(text.len() - 3);
                self.text_color(x, y + row, &format!("{}...", text.0), text_color, bg_color);
            }
        }
    }

    pub fn text_box(&mut self, x: u32, y: u32, width: u32, height: u32, text: &str) {
        let text = textwrap::wrap(text, Options::new(width as usize));

        for row in 0..height {
            if text.len() <= row as usize {
                break;
            }

            let line = &text[row as usize];

            if row != (height - 1) {
                self.text(x, y + row, &line);
            } else {
                let text = line.split_at(text.len() - 3);
                self.text(x, y + row, &format!("{}...", text.0));
            }
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
    buffer: &'b mut AsciiBuffer,
    bg_color: Color,
    border_color: Color,
    title_color: Color,
    title_bg_color: Option<Color>,
    with_border: bool,
    title: Option<String>,
    border: BorderType,
}

impl<'b> AsciiBoxDrawer<'b> {
    pub fn draw(mut self) -> AsciiBuffer {
        for y in 0..self.buffer.height {
            for x in 0..self.buffer.width {}
        }
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

pub enum TextOverflow {
    Hidden,
    Wrap,
    Elipses,
    None,
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
        buffer.filled_border_box(buffer.width - 21, 1, 20, 11, Color::White, Color::LightBlue);
        buffer.text_color(
            buffer.width - 20,
            2,
            "Hello World",
            Color::White,
            Color::LightBlue,
        );
        buffer.text_box(
            buffer.width - 20,
            3,
            18,
            10,
            "This is a test for a long format text box. I am hoping that this works.",
        )
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