use std::sync::{Arc, Mutex};
use bevy::{ecs::component::Component, reflect::Reflect};

use super::{bounds::AsciiBounds, character::{AsciiCharacter, Color}, BorderType, Character, HorizontalAlignment, Padding, TextOverflow, VerticalAlignment};

//=============================================================================
//             Ascii Buffer
//=============================================================================

#[derive(Clone)]
pub struct AsciiBuffer {
    surface: AsciiSurface,
    pub bounds : AsciiBounds,
}

impl AsciiBuffer {
    pub fn new(surface : &AsciiSurface, bounds : &AsciiBounds) -> Self {
       AsciiBuffer {
           surface: surface.clone(),
           bounds: bounds.clone(),
       }
    }
    
    pub fn with_layer(mut self, layer : u32) -> Self {
       self.bounds.layer = layer;
       self
    }
    
    pub fn set_character(&self, x: u32, y: u32, character: impl Into<AsciiCharacter>) {
        if self.bounds.is_within_local(x, y) {
            let character = character.into().with_layer(self.bounds.layer);
            self.surface.set_character(self.bounds.x + x, self.bounds.y + y, character);
        }
    }

    pub fn sub_buffer(&self, x: u32, y: u32, width: u32, height: u32) -> Option<AsciiBuffer> {
        if self.bounds.is_within(x, y) {
            // let width = self.bounds.width.saturating_sub(x).min(width);
            // let height = self.bounds.height.saturating_sub(y).min(height);

            return Some(AsciiBuffer {
                surface: self.surface.clone(),
                bounds : AsciiBounds::new(x, y, width, height),
            });
        }

        None
    }
    
    pub fn top(&self, size : u32) -> (AsciiBuffer, Option<AsciiBuffer>) {
        let mut top = self.clone();
        if size < self.bounds.height {
            let mut bottom = self.clone();
            top.bounds.height = size;
            bottom.bounds.y += size;
            bottom.bounds.height -= size;
            (top, Some(bottom))
        } else {
            (top, None)
        }
    }

    pub fn center(&self, width: u32, height: u32) -> AsciiBuffer {
        AsciiBuffer {
            surface: self.surface.clone(),
            bounds : AsciiBounds::new(
                ((self.bounds.width / 2) - (width / 2)).max(0),
                ((self.bounds.height / 2) - (height / 2)).max(0),
                width.min(self.bounds.width),
                height.min(self.bounds.height),
            ),
        }
    }
    
    pub fn vertical_split<const COUNT : usize>(&self) -> Option<[AsciiBuffer; COUNT]> {
        let width = self.bounds.width / COUNT as u32;
        if width == 0 {
            return None;
        }
        let mut buffers = Vec::new();
        let mut x = 0;
        for _ in 0..COUNT {
            let buffer = AsciiBuffer {
                surface: self.surface.clone(),
                bounds : AsciiBounds::new(self.bounds.x + x, self.bounds().y, width, self.bounds.height),
            };
            buffers.push(buffer);
            x += width;
        }
        
        if let Ok(buffer) = buffers.try_into() {
            Some(buffer)
        } else {
            None
        }
    }
    
    pub fn horizontal_split<const COUNT : usize>(&self) -> Option<[AsciiBuffer; COUNT]> {
        let height = self.bounds.width / COUNT as u32;
        if height == 0 {
            return None;
        }
        let mut buffers = Vec::new();
        let mut y = 0;
        for _ in 0..COUNT {
            let buffer = AsciiBuffer {
                surface: self.surface.clone(),
                bounds : AsciiBounds::new(self.bounds.x, self.bounds.y + y, self.bounds.width, height),
            };
            buffers.push(buffer);
            y += height;
        }
        
        if let Ok(buffer) = buffers.try_into() {
            Some(buffer)
        } else {
            None
        }
    }
    
    pub fn padding(&self, padding : impl Into<Padding>) -> AsciiBuffer {
        let padding = padding.into();
        let mut buffer = self.clone();
        let horizontal_difference = buffer.bounds.width as i32 - padding.left as i32 - padding.right as i32;
        let vertical_difference = buffer.bounds.height as i32 - padding.top as i32 - padding.bottom as i32;
        if horizontal_difference <= 0 || vertical_difference <= 0 {
            return buffer;
        }
        buffer.bounds.x += padding.left;
        buffer.bounds.y += padding.top;
        buffer.bounds.width -= padding.left + padding.right;
        buffer.bounds.height -= padding.top + padding.bottom;
        buffer
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
    
    pub fn text(&self, text : &str) -> AsciiTextDrawer {
        AsciiTextDrawer {
            buffer: self,
            text_color: Color::White,
            bg_color: Color::Black,
            text: text.to_string(),
            horizontal_alignment: HorizontalAlignment::Left,
            vertical_alignment: VerticalAlignment::Top,
            overflow: TextOverflow::default(),
            should_wrap: false
        }
    }
    
    pub fn bounds(&self) -> &AsciiBounds {
        &self.bounds
    }
}

//=============================================================================
//             Ascii UiSurface
//=============================================================================

#[derive(Clone)]
pub struct AsciiSurface {
    width : u32,
    height : u32,
    data : Arc<Mutex<Vec<AsciiCharacter>>>,
}

impl Default for AsciiSurface {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            data: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl AsciiSurface {
    pub fn new(width : u32, height : u32) -> Self {
        let data = vec![AsciiCharacter::default(); (width * height) as usize];
        Self { width, height, data : Arc::new(Mutex::new(data)) }
    }
    
    pub fn set_character(&self, x : u32, y : u32, character : AsciiCharacter) {
        let Ok(mut data) = self.data.lock() else {return};
        let index = self.calc_index(x, y);
        if index < data.len() {
            let data = &mut data[index];
            let layer_test = match (&character, &data) {
                (
                    AsciiCharacter::Set { index : _, text_color : _, background_color : _, layer : input_layer }, 
                    AsciiCharacter::Set { index : _, text_color : _, background_color : _, layer : data_layer }
                ) => {
                    input_layer >= data_layer
                },
                (AsciiCharacter::Set { index : _, text_color : _, background_color : _, layer : _ }, AsciiCharacter::Unset) => true,
                (AsciiCharacter::Unset, AsciiCharacter::Set { index : _, text_color : _, background_color : _, layer : _ }) => true,
                (AsciiCharacter::Unset, AsciiCharacter::Unset) => false,
            };
            
            if layer_test {
                *data = character;
            }
        }
    }
    
    fn calc_index(&self, x: u32, y: u32) -> usize {
        (x + (y * self.width)) as usize
    }
    
    pub fn as_byte_vec(&self) -> Vec<u8> {
        let result = self
            .data
            .lock()
            .expect("Error while writing surface: data is poisoned.")
            .iter()
            .map(|value| value.into_u8())
            .flatten()
            .collect();
        result
    }
    
    pub fn len(&self) -> usize {
        (self.width * self.height) as usize
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
        for y in 0..self.buffer.bounds.height {
            for x in 0..self.buffer.bounds.width {
                let character = self.calc_character(x, y);
                self.buffer.set_character(x, y, character);
            }
        }

        self.buffer
            .sub_buffer(
                self.buffer.bounds.x + 1, 
                self.buffer.bounds.y + 1, 
                self.buffer.bounds.width.saturating_sub(2), 
                self.buffer.bounds.height.saturating_sub(2)
            )
    }

    fn calc_character(&mut self, x: u32, y: u32) -> AsciiCharacter {
        let max_title_width = self.buffer.bounds.width as i32 - 4;
        let character = self
            .border
            .get_character(x, y, self.buffer.bounds.width, self.buffer.bounds.height);
        if max_title_width < 2 {
            return (character, self.border_color, self.bg_color).into();
        }

        if let Some(title) = &self.title {
            if y == 0 && x >= 2 && x <= self.buffer.bounds.width - 2 {
                let title_len = title.len().min(max_title_width as usize);
                // let difference = title_len as i32 - max_title_width;
                let x_start = match self.title_alignment {
                    HorizontalAlignment::Left => 2,
                    HorizontalAlignment::Center => {
                        (self.buffer.bounds.width / 2 - title_len as u32 / 2).max(2)
                    }
                    HorizontalAlignment::Right => {
                        (self.buffer.bounds.width as i32 - title_len as i32 - 2).max(2) as u32
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

//=============================================================================
//             Ascii Text Drawer
//=============================================================================

pub struct AsciiTextDrawer<'b> {
    buffer: &'b AsciiBuffer,
    text: String,
    text_color: Color,
    bg_color: Color,
    horizontal_alignment: HorizontalAlignment,
    vertical_alignment: VerticalAlignment,
    overflow: TextOverflow,
    should_wrap: bool,
}

impl <'b> AsciiTextDrawer<'b> {
    
    pub fn draw(self) {
        let lines : Vec<String> = if self.should_wrap {
            let lines = textwrap::wrap(self.text.as_str(), self.buffer.bounds.width as usize);
            lines.iter().map(|s| s.to_string()).collect()
        } else {
            self.text.lines().map(|s| s.to_string()).collect::<Vec<String>>()
        };
        
        for line in 0..self.buffer.bounds.height as usize {
            let Some(text) = lines.get(line) else {break};
            
            let start_x = match self.horizontal_alignment {
                HorizontalAlignment::Left => 0,
                HorizontalAlignment::Center => (self.buffer.bounds.width as f32 / 2.0 - text.len() as f32 / 2.0).floor() as u32,
                HorizontalAlignment::Right => self.buffer.bounds.width - text.len() as u32 - 1,
            };
            
            let start_y = match self.vertical_alignment {
                VerticalAlignment::Top => 0,
                VerticalAlignment::Center => (self.buffer.bounds.height as f32 / 2.0 - lines.len() as f32 / 2.0).floor() as u32,
                VerticalAlignment::Bottom => self.buffer.bounds.height - lines.len() as u32 - 1,
            };
            
            for column in 0..(self.buffer.bounds.width - 1) as usize {
                if let Some(c) = text.chars().nth(column) {
                    self.buffer.set_character(
                        start_x + column as u32, 
                        start_y + line as u32, 
                        (c, self.text_color, self.bg_color)
                    );
                }
            }
        }
    }
    
    pub fn bg_color(mut self, bg_color: Color) -> Self {
        self.bg_color = bg_color;
        self
    }
    
    pub fn text_color(mut self, text_color: Color) -> Self {
        self.text_color = text_color;
        self
    }
    
    pub fn horizontal_alignment(mut self, alignment: HorizontalAlignment) -> Self {
        self.horizontal_alignment = alignment;
        self
    }
    
    pub fn overflow(mut self, overflow: TextOverflow) -> Self {
        self.overflow = overflow;
        self
    }
    
    pub fn wrap(mut self) -> Self {
        self.should_wrap = true;
        self
    }
    
    pub fn vertical_alignment(mut self, alignment: VerticalAlignment) -> Self {
        self.vertical_alignment = alignment;
        self
    }
}