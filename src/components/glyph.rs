use bracket_lib::prelude::{RGB, RGBA};
use serde::de::Visitor;
use serde::{Deserialize, Deserializer};

#[derive(Debug, Deserialize, Default, Copy, Clone)]
pub struct Glyph {
    pub character: char,
    pub colour: Colour,
}

#[derive(Debug, Default, Copy, Clone)]
pub struct Colour {
    pub rgba: RGBA,
}

impl<T: Into<RGBA>> From<T> for Colour {
    fn from(rgba: T) -> Self {
        Colour { rgba: rgba.into() }
    }
}

struct ColourVisitor;
impl<'de> Visitor<'de> for ColourVisitor {
    type Value = Colour;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Error")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> {
        Ok(RGBA::from_hex(value)
            .unwrap_or_else(|_| {
                RGB::from_hex(value)
                    .expect("Expected hex colour (eg: \"#ffffff\")")
                    .into()
            })
            .into())
    }
}

impl<'de> Deserialize<'de> for Colour {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(deserializer.deserialize_str(ColourVisitor)?)
    }
}
