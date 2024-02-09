//=============================================================================
//             Ascii Buffer
//=============================================================================

use std::{rc::Rc, sync::Mutex};

use super::{AsciiCharacter, BorderType, Character, Color, HorizontalAlignment, TextOverflow};

pub struct AsciiBuffer {
    surface: Rc<Mutex<Vec<AsciiCharacter>>>,
    surface_width: u32,
    surface_height: u32,
    width: u32,
    height: u32,
    x: u32,
    y: u32,
}

impl AsciiBuffer {
    pub fn new(width : u32, height : u32) -> Self {
       let data = vec![AsciiCharacter::default(); (width * height) as usize];
       AsciiBuffer {
           surface: Rc::new(Mutex::new(data)),
           width, 
           height,
           x: 0,
           y: 0,
           surface_width: width,
           surface_height: height,
       }
    }
    
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

    pub fn as_byte_vec(&self) -> Vec<u8> {
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