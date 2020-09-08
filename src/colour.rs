use palette::{FromColor, IntoColor, LinSrgb};
use serde::de::{Deserialize, Deserializer, Error as _, MapAccess, Visitor};
use std::{convert::TryFrom, fmt};

#[derive(Debug, Copy, Clone, serde::Deserialize)]
#[serde(try_from = "&str")]
pub struct Colour {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Clone)]
pub enum Gradient {
    Rgb(palette::Gradient<palette::LinSrgb<f64>>),
    Hsv(palette::Gradient<palette::Hsv<palette::encoding::srgb::Srgb, f64>>),
}

impl Gradient {
    pub fn get(&self, i: f64) -> Colour {
        let c = match &self {
            Gradient::Rgb(g) => g.get(i),
            Gradient::Hsv(g) => g.get(i).into_rgb(),
        };
        Colour::from(c)
    }
}

impl From<LinSrgb<f64>> for Colour {
    fn from(c: LinSrgb<f64>) -> Self {
        Colour {
            r: (c.red * 255.0) as _,
            g: (c.green * 255.0) as _,
            b: (c.blue * 255.0) as _,
        }
    }
}

impl From<Colour> for LinSrgb<f64> {
    fn from(c: Colour) -> Self {
        LinSrgb::new(c.r as f64 / 255.0, c.g as f64 / 255.0, c.b as f64 / 255.0)
    }
}

impl TryFrom<&str> for Colour {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if !value.starts_with('#') || value.len() != 7 {
            return Err(anyhow::anyhow!("colours should be in #rrggbb format"));
        }

        let r = u8::from_str_radix(&value[1..3], 16)?;
        let g = u8::from_str_radix(&value[3..5], 16)?;
        let b = u8::from_str_radix(&value[5..7], 16)?;

        Ok(Self { r, g, b })
    }
}

impl Default for Gradient {
    fn default() -> Self {
        Self::Rgb(palette::Gradient::new(
            [
                Colour { r: 5, g: 10, b: 60 },
                Colour {
                    r: 140,
                    g: 40,
                    b: 70,
                },
                Colour {
                    r: 240,
                    g: 200,
                    b: 60,
                },
                Colour { r: 5, g: 10, b: 60 },
            ]
            .iter()
            .copied()
            .map(<LinSrgb<f64>>::from),
        ))
    }
}

impl<'de> Deserialize<'de> for Gradient {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Mode,
            Colours,
        }

        #[derive(serde::Deserialize)]
        #[serde(rename_all = "UPPERCASE")]
        enum Mode {
            Rgb,
            Hsv,
        }

        struct GradientVisitor;

        impl<'de> Visitor<'de> for GradientVisitor {
            type Value = Gradient;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a gradient mode and a list of colours")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, <A as MapAccess<'de>>::Error>
            where
                A: MapAccess<'de>,
            {
                let mut mode = None;
                let mut colours: Option<Vec<Colour>> = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Mode => {
                            if mode.is_some() {
                                return Err(<A as MapAccess<'de>>::Error::duplicate_field("mode"));
                            }
                            mode = Some(map.next_value()?);
                        }
                        Field::Colours => {
                            if colours.is_some() {
                                return Err(<A as MapAccess<'de>>::Error::duplicate_field(
                                    "colours",
                                ));
                            }
                            colours = Some(map.next_value()?);
                        }
                    }
                }

                let mode = mode.unwrap_or(Mode::Rgb);
                let colours = colours
                    .ok_or_else(|| <A as MapAccess<'de>>::Error::missing_field("colours"))?;

                let lin_srgb = colours.into_iter().map(<LinSrgb<f64>>::from);

                match mode {
                    Mode::Rgb => Ok(Gradient::Rgb(palette::Gradient::new(lin_srgb))),
                    Mode::Hsv => Ok(Gradient::Hsv(palette::Gradient::new(
                        lin_srgb.map(FromColor::from_rgb),
                    ))),
                }
            }
        }

        const FIELDS: &[&str] = &["mode", "colours"];
        deserializer.deserialize_struct("Gradient", FIELDS, GradientVisitor)
    }
}