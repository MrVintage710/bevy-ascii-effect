use bevy::{ecs::component::Component, reflect::Reflect};
use std::sync::{Arc, Mutex};

use super::{
    bounds::AsciiBounds, character::{AsciiCharacter, Color}, position::AsciiPosition, util::Value, BorderType, Character, HorizontalAlignment, Padding, TextOverflow, VerticalAlignment
};

//=============================================================================
//             Ascii Buffer
//=============================================================================

#[derive(Clone)]
pub struct AsciiBuffer {
    surface: AsciiSurface,
    pub bounds: AsciiBounds,
    clip_bounds: Option<AsciiBounds>,
}

impl AsciiBuffer {
    pub fn new(surface: &AsciiSurface, bounds: &AsciiBounds, clip_bounds : Option<AsciiBounds>) -> Self {
        AsciiBuffer {
            surface: surface.clone(),
            bounds: bounds.clone(),
            clip_bounds,
        }
    }
    
    pub fn clip(&self) -> AsciiBuffer {
        AsciiBuffer {
            surface: self.surface.clone(),
            bounds: self.bounds.clone(),
            clip_bounds: Some(self.bounds.clone()),
        }
    }

    pub fn set_character(&self, x: i32, y: i32, character: impl Into<AsciiCharacter>) {
        let x = self.bounds.x + x;
        let y = self.bounds.y + y;
        
        if let Some(clip_bounds) = &self.clip_bounds {
            if !clip_bounds.is_within(x, y) {
                return;
            }
        }
        
        let character = character.into().with_layer(self.bounds.layer);
        self.surface
            .set_character(x, y, character);
            
    }

    pub fn sub_buffer(&self, x: i32, y: i32, width: u32, height: u32) -> Option<AsciiBuffer> {
        if self.bounds.is_within(x, y) {
            // let width = self.bounds.width.saturating_sub(x).min(width);
            // let height = self.bounds.height.saturating_sub(y).min(height);

            return Some(AsciiBuffer {
                surface: self.surface.clone(),
                bounds: AsciiBounds::new(x, y, width, height, self.bounds.layer + 1),
                clip_bounds: self.clip_bounds.clone(),
            });
        }

        None
    }
    
    pub fn relative(&self, x: i32, y: i32, width: impl Into<Value>, height: impl Into<Value>) -> AsciiBuffer {
        let mut child_bounds = AsciiBounds::default();
        AsciiPosition::relative(x, y, width, height, self.bounds.layer).format_bounds(self.bounds(), &mut child_bounds);
        AsciiBuffer {
            surface: self.surface.clone(),
            bounds: child_bounds,
            clip_bounds: self.clip_bounds.clone(),
        }
    }

    pub fn top(&self, size: u32) -> AsciiBuffer {
        let mut child_bounds = AsciiBounds::default();
        AsciiPosition::top(size as i32).format_bounds(self.bounds(), &mut child_bounds);
        AsciiBuffer {
            surface: self.surface.clone(),
            bounds: child_bounds,
            clip_bounds: self.clip_bounds.clone(),
        }
    }
    
    pub fn bottom(&self, size: u32) -> AsciiBuffer {
        let mut child_bounds = AsciiBounds::default();
        AsciiPosition::bottom(size as i32).format_bounds(self.bounds(), &mut child_bounds);
        AsciiBuffer {
            surface: self.surface.clone(),
            bounds: child_bounds,
            clip_bounds: self.clip_bounds.clone(),
        }
    }
    
    pub fn left(&self, size: u32) -> AsciiBuffer {
        let mut child_bounds = AsciiBounds::default();
        AsciiPosition::left(size as i32).format_bounds(self.bounds(), &mut child_bounds);
        AsciiBuffer {
            surface: self.surface.clone(),
            bounds: child_bounds,
            clip_bounds: self.clip_bounds.clone(),
        }
    }
    
    pub fn right(&self, size: u32) -> AsciiBuffer {
        let mut child_bounds = AsciiBounds::default();
        AsciiPosition::right(size as i32).format_bounds(self.bounds(), &mut child_bounds);
        AsciiBuffer {
            surface: self.surface.clone(),
            bounds: child_bounds,
            clip_bounds: self.clip_bounds.clone(),
        }
    }

    pub fn center(&self, width: impl Into<Value>, height: impl Into<Value>) -> AsciiBuffer {
        let mut child_bounds = AsciiBounds::default();
        AsciiPosition::format_bounds_aligned(width, height, HorizontalAlignment::Center, VerticalAlignment::Center, self.bounds(), &mut child_bounds);
        AsciiBuffer {
            surface: self.surface.clone(),
            bounds: child_bounds,
            clip_bounds: self.clip_bounds.clone(),
        }
    }

    // pub fn vertical_split<const COUNT: usize>(&self) -> Option<[AsciiBuffer; COUNT]> {
    //     let width = self.bounds.width / COUNT as u32;
    //     if width == 0 {
    //         return None;
    //     }
    //     let mut buffers = Vec::new();
    //     let mut x = 0;
    //     for _ in 0..COUNT {
    //         let buffer = AsciiBuffer {
    //             surface: self.surface.clone(),
    //             bounds: AsciiBounds::new(
    //                 self.bounds.x + x,
    //                 self.bounds().y,
    //                 width,
    //                 self.bounds.height,
    //                 self.bounds.layer + 1,
    //             ),
    //             should_clip: self.should_clip.clone()
    //         };
    //         buffers.push(buffer);
    //         x += width as i32;
    //     }

    //     if let Ok(buffer) = buffers.try_into() {
    //         Some(buffer)
    //     } else {
    //         None
    //     }
    // }

    // pub fn horizontal_split<const COUNT: usize>(&self) -> Option<[AsciiBuffer; COUNT]> {
    //     let height = self.bounds.width / COUNT as u32;
    //     if height == 0 {
    //         return None;
    //     }
    //     let mut buffers = Vec::new();
    //     let mut y = 0;
    //     for _ in 0..COUNT {
    //         let buffer = AsciiBuffer {
    //             surface: self.surface.clone(),
    //             bounds: AsciiBounds::new(
    //                 self.bounds.x,
    //                 self.bounds.y + y,
    //                 self.bounds.width,
    //                 height,
    //                 self.bounds.layer + 1,
    //             ),
    //             should_clip: self.should_clip
    //         };
    //         buffers.push(buffer);
    //         y += height as i32;
    //     }

    //     if let Ok(buffer) = buffers.try_into() {
    //         Some(buffer)
    //     } else {
    //         None
    //     }
    // }

    pub fn padding(&self, padding: impl Into<Padding>) -> AsciiBuffer {
        let padding = padding.into();
        let mut buffer = self.clone();
        let horizontal_difference =
            buffer.bounds.width as i32 - padding.left as i32 - padding.right as i32;
        let vertical_difference =
            buffer.bounds.height as i32 - padding.top as i32 - padding.bottom as i32;
        if horizontal_difference <= 0 || vertical_difference <= 0 {
            return buffer;
        }
        buffer.bounds.x += padding.left as i32;
        buffer.bounds.y += padding.top as i32;
        buffer.bounds.width -= padding.left + padding.right;
        buffer.bounds.height -= padding.top + padding.bottom;
        buffer
    }
    
    pub fn border(&self, border_type : BorderType) -> AsciiBorderDrawer {
        AsciiBorderDrawer {
            buffer: self,
            border_color: Color::White,
            bg_color: Color::Black,
            border_type,
            top: false,
            bottom: false,
            left: false,
            right: false,
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

    pub fn text(&self, text: &str) -> AsciiTextDrawer {
        AsciiTextDrawer {
            buffer: self,
            text_color: Color::White,
            bg_color: Color::Black,
            text: text.to_string(),
            horizontal_alignment: HorizontalAlignment::Left,
            vertical_alignment: VerticalAlignment::Top,
            overflow: TextOverflow::default(),
            should_wrap: false,
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
    width: u32,
    height: u32,
    data: Arc<Mutex<Vec<AsciiCharacter>>>,
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
    pub fn new(width: u32, height: u32) -> Self {
        let data = vec![AsciiCharacter::default(); (width * height) as usize];
        Self {
            width,
            height,
            data: Arc::new(Mutex::new(data)),
        }
    }

    pub fn set_character(&self, x: i32, y: i32, character: AsciiCharacter) {
        let Ok(mut data) = self.data.lock() else {
            return;
        };
        let Some(index) = self.calc_index(x, y) else {
            return;
        };
        if index < data.len() {
            let data = &mut data[index];
            let layer_test = match (&character, &data) {
                (
                    AsciiCharacter::Set {
                        index: _,
                        text_color: _,
                        background_color: _,
                        layer: input_layer,
                    },
                    AsciiCharacter::Set {
                        index: _,
                        text_color: _,
                        background_color: _,
                        layer: data_layer,
                    },
                ) => input_layer >= data_layer,
                (
                    AsciiCharacter::Set {
                        index: _,
                        text_color: _,
                        background_color: _,
                        layer: _,
                    },
                    AsciiCharacter::Unset,
                ) => true,
                (
                    AsciiCharacter::Unset,
                    AsciiCharacter::Set {
                        index: _,
                        text_color: _,
                        background_color: _,
                        layer: _,
                    },
                ) => true,
                (AsciiCharacter::Unset, AsciiCharacter::Unset) => false,
            };

            if layer_test {
                *data = character;
            }
        }
    }

    fn calc_index(&self, x: i32, y: i32) -> Option<usize> {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return None;
        } else {
            Some((x as u32 + (y as u32 * self.width)) as usize)
        }
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
    
    pub fn clear(&self) {
        self.data.lock().expect("Error while clearing surface: data is poisoned.").clear();
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
                self.buffer.set_character(x as i32, y as i32, character);
            }
        }

        self.buffer.sub_buffer(
            self.buffer.bounds.x + 1,
            self.buffer.bounds.y + 1,
            self.buffer.bounds.width.saturating_sub(2),
            self.buffer.bounds.height.saturating_sub(2),
        )
    }

    fn calc_character(&mut self, x: u32, y: u32) -> AsciiCharacter {
        let max_title_width = self.buffer.bounds.width as i32 - 4;
        let character =
            self.border
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
        let lines: Vec<String> = if self.should_wrap {
            let lines = textwrap::wrap(self.text.as_str(), self.buffer.bounds.width as usize);
            lines.iter().map(|s| s.to_string()).collect()
        } else {
            self.text
                .lines()
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
        };

        for line in 0..self.buffer.bounds.height as usize {
            let Some(text) = lines.get(line) else { break };

            let start_x = match self.horizontal_alignment {
                HorizontalAlignment::Left => 0,
                HorizontalAlignment::Center => {
                    (self.buffer.bounds.width as f32 / 2.0 - text.len() as f32 / 2.0).floor() as i32
                }
                HorizontalAlignment::Right => {
                    self.buffer.bounds.width as i32 - text.len() as i32 - 1
                }
            };

            let start_y = match self.vertical_alignment {
                VerticalAlignment::Top => 0,
                VerticalAlignment::Center => (self.buffer.bounds.height as f32 / 2.0
                    - lines.len() as f32 / 2.0)
                    .floor() as i32,
                VerticalAlignment::Bottom => {
                    self.buffer.bounds.height as i32 - lines.len() as i32 - 1
                }
            };

            for column in 0..(self.buffer.bounds.width - 1) as usize {
                if let Some(c) = text.chars().nth(column) {
                    self.buffer.set_character(
                        start_x + column as i32,
                        start_y + line as i32,
                        (c, self.text_color, self.bg_color),
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

//=============================================================================
//             Border Drawer
//=============================================================================

pub struct AsciiBorderDrawer<'b> {
    buffer : &'b AsciiBuffer,
    border_color : Color,
    bg_color : Color,
    border_type : BorderType,
    top : bool,
    bottom : bool,
    left : bool,
    right : bool,
}

impl <'b> AsciiBorderDrawer<'b> {
    pub fn draw(self) -> Option<AsciiBuffer> {
        let width = self.buffer.bounds.width;
        let height = self.buffer.bounds.height;
        
        if self.top {
            let character = self.border_type.top();
            for x in 0..width {
                self.buffer.set_character(x as i32, 0, (character, Color::White, Color::Black));
            }
        }

        if self.bottom {
            let character = self.border_type.bottom();
            for x in 0..width {
                self.buffer.set_character(x as i32, height as i32, (character, Color::White, Color::Black));
            }
        }

        if self.left {
            let character = self.border_type.left();
            for y in 0..height {
                self.buffer.set_character(0, y as i32, (character, Color::White, Color::Black));
            }
        }

        if self.right {
            let character = self.border_type.right();
            for y in 0..height {
                self.buffer.set_character(width as i32 - 1, y as i32, (character, Color::White, Color::Black));
            }
        }
        
        let new_width = self.buffer.bounds.width - if self.left { 1 } else { 0 } - if self.right { 1 } else { 0 };
        let new_height = self.buffer.bounds.height - if self.top { 1 } else { 0 } - if self.bottom { 1 } else { 0 };
        let new_x = 0 + if self.left { 1 } else { 0 };
        let new_y = 0 + if self.top { 1 } else { 0 };
        self.buffer.sub_buffer(new_x, new_y, new_width, new_height)
    }

    pub fn all(mut self) -> Self {
        self.top = true;
        self.bottom = true;
        self.left = true;
        self.right = true;
        self
    }
    
    pub fn vertical(mut self) -> Self {
        self.left = true;
        self.right = true;
        self
    }
    
    pub fn horizontal(mut self) -> Self {
        self.top = true;
        self.bottom = true;
        self
    }
    
    pub fn top(mut self) -> Self {
        self.top = true;
        self
    }

    pub fn bottom(mut self) -> Self {
        self.bottom = true;
        self
    }

    pub fn left(mut self) -> Self {
        self.left = true;
        self
    }

    pub fn right(mut self) -> Self {
        self.right = true;
        self
    }
    
    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = color;
        self
    }
    
    pub fn bg_color(mut self, color: Color) -> Self {
        self.bg_color = color;
        self
    }
}