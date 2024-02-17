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