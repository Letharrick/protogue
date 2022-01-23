use bracket_lib::prelude::{BTerm, RGB, WHITE};

use components::glyph::{Colour, Glyph};
use std::ops::{Add, AddAssign};

// A fragment of text within a label
pub struct Fragment {
    pub text: String,
    pub colour: Option<Colour>,
}

impl Fragment {
    pub fn new<S: ToString>(text: S, colour: Option<Colour>) -> Self {
        Self {
            text: text.to_string(),
            colour,
        }
    }
}

// &str -> Fragment
impl From<&str> for Fragment {
    fn from(string: &str) -> Self {
        Self {
            text: string.to_string(),
            colour: None,
        }
    }
}

// String -> Fragment
impl From<String> for Fragment {
    fn from(string: String) -> Self {
        Self {
            text: string.to_string(),
            colour: None,
        }
    }
}

// (&str, Into<RGB>) -> Fragment
impl<C: Into<RGB>> From<(&str, C)> for Fragment {
    fn from((string, colour): (&str, C)) -> Self {
        Self {
            text: string.to_string(),
            colour: Some(colour.into().into()),
        }
    }
}

// (&str, Colour) -> Fragment
impl From<(&str, Colour)> for Fragment {
    fn from((string, colour): (&str, Colour)) -> Self {
        Self {
            text: string.to_string(),
            colour: Some(colour),
        }
    }
}

impl<C: Into<RGB>> From<(String, C)> for Fragment {
    fn from((string, colour): (String, C)) -> Self {
        Self {
            text: string,
            colour: Some(colour.into().into()),
        }
    }
}

impl From<(String, Colour)> for Fragment {
    fn from((string, colour): (String, Colour)) -> Self {
        Self {
            text: string,
            colour: Some(colour),
        }
    }
}

impl From<Glyph> for Fragment {
    fn from(glyph: Glyph) -> Self {
        (glyph.character.to_string(), glyph.colour).into()
    }
}

// A label strung together with fragments
pub struct Label {
    pub fragments: Vec<Fragment>,
}

impl Label {
    pub fn new() -> Self {
        Self {
            fragments: Vec::default(),
        }
    }
}

// Label concatenation
impl Add<Label> for Label {
    type Output = Label;

    fn add(mut self, rhs: Label) -> Self::Output {
        self.fragments.extend(rhs.fragments);

        self
    }
}

impl AddAssign<Label> for Label {
    fn add_assign(&mut self, rhs: Label) {
        self.fragments.extend(rhs.fragments);
    }
}

impl From<&str> for Label {
    fn from(string: &str) -> Self {
        Self {
            fragments: vec![string.into()],
        }
    }
}

pub trait Element {
    fn render(&self, ctx: &mut BTerm);
    fn position(&self) -> (i32, i32);
    fn scale(&self) -> (i32, i32);
    fn foreground_colour(&self) -> (u8, u8, u8) {
        WHITE
    }
    fn background_colour(&self) -> (u8, u8, u8) {
        (10, 10, 15)
    }
}

pub struct List {
    position: (i32, i32),
    scale: (i32, i32),
    entries: Vec<Label>,
    entry_padding: i32,
}

impl List {
    pub fn new(position: (i32, i32), scale: (i32, i32), entry_padding: i32) -> Self {
        Self {
            position,
            scale,
            entries: Vec::default(),
            entry_padding,
        }
    }

    pub fn add(&mut self, element: Label) {
        self.entries.push(element);
    }

    pub fn remove<T: ToString>(&mut self, element: T) {
        if let Some(index) = self.entries.iter().position(|label| {
            label
                .fragments
                .iter()
                .fold(String::new(), |mut string, fragment| {
                    string += fragment.text.as_str();

                    string
                })
                == element.to_string()
        }) {
            self.entries.remove(index);
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

impl Element for List {
    fn render(&self, ctx: &mut BTerm) {
        ctx.draw_box(
            self.position.0 as i32,
            self.position.1 as i32,
            self.scale.0,
            self.scale.1,
            self.foreground_colour(),
            self.background_colour(),
        );

        for (index, label) in self.entries.iter().enumerate() {
            let mut x = self.position.0 + 1;
            let y = self.position.1 + self.entry_padding + index as i32 + 1;
            for fragment in &label.fragments {
                match fragment.colour {
                    None => ctx.print(x, y, &fragment.text),
                    Some(colour) => {
                        ctx.print_color(x, y, colour.rgba, RGB::from((0, 0, 0)), &fragment.text)
                    }
                }

                x += fragment.text.len() as i32;
            }
        }
    }

    fn position(&self) -> (i32, i32) {
        self.position
    }
    fn scale(&self) -> (i32, i32) {
        self.scale
    }
}

#[macro_export]
macro_rules! label {
    [$($fragment:expr),*] => {{
        let mut label = Label::new();

        $(
            label.fragments.push($fragment.into());
        )*

        label
    }};
}
