/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Specified types for CSS values related to effects.

use crate::Zero;
use crate::derives::*;
use crate::parser::{Parse, ParserContext};
#[cfg(all(feature = "servo", not(feature = "ahcut")))]
use crate::values::Impossible;
use crate::values::computed::Angle as ComputedAngle;
use crate::values::computed::CSSPixelLength as ComputedCSSPixelLength;
use crate::values::computed::Filter as ComputedFilter;
use crate::values::computed::NonNegativeLength as ComputedNonNegativeLength;
use crate::values::computed::NonNegativeNumber as ComputedNonNegativeNumber;
use crate::values::computed::Number as ComputedNumber;
use crate::values::computed::NumberOrPercentage as ComputedNumberOrPercentage;
use crate::values::computed::ZeroToOneNumber as ComputedZeroToOneNumber;
use crate::values::computed::effects::BoxShadow as ComputedBoxShadow;
use crate::values::computed::effects::SimpleShadow as ComputedSimpleShadow;
#[cfg(any(feature = "gecko", feature = "ahcut"))]
use crate::values::computed::url::ComputedUrl;
use crate::values::computed::{Context, ToComputedValue};
use crate::values::generics::effects::BoxShadow as GenericBoxShadow;
use crate::values::generics::effects::Filter as GenericFilter;
use crate::values::generics::effects::SimpleShadow as GenericSimpleShadow;
use crate::values::generics::{NonNegative, ZeroToOne};
use crate::values::specified::color::Color;
use crate::values::specified::length::{Length, NonNegativeLength};
#[cfg(any(feature = "gecko", feature = "ahcut"))]
use crate::values::specified::url::SpecifiedUrl;
use crate::values::specified::{Angle, NonNegativeNumberOrPercentage, Number, NumberOrPercentage};
use cssparser::{Parser, match_ignore_ascii_case};
use style_traits::{ParseError, StyleParseErrorKind};

/// A specified value for a single shadow of the `box-shadow` property.
pub type BoxShadow =
    GenericBoxShadow<Option<Color>, Length, Option<NonNegativeLength>, Option<Length>>;

/// A specified value for a single `filter`.
#[cfg(any(feature = "gecko", feature = "ahcut"))]
pub type SpecifiedFilter = GenericFilter<Angle, FilterFactor, Length, SimpleShadow, SpecifiedUrl>;

/// A specified value for a single `filter`.
#[cfg(all(feature = "servo", not(feature = "ahcut")))]
pub type SpecifiedFilter = GenericFilter<Angle, FilterFactor, Length, SimpleShadow, Impossible>;

pub use self::SpecifiedFilter as Filter;

/// The factor for a filter function.
#[derive(Clone, Debug, MallocSizeOf, PartialEq, SpecifiedValueInfo, ToCss, ToShmem)]
pub struct FilterFactor(NumberOrPercentage);

impl FilterFactor {
    fn to_computed_value_without_context(&self) -> Result<ComputedNumberOrPercentage, ()> {
        self.0.to_computed_value_without_context()
    }
}

impl ToComputedValue for FilterFactor {
    type ComputedValue = ComputedNumber;
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        self.0.to_computed_value(context).value()
    }
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        Self(NumberOrPercentage::Number(Number::new(*computed)))
    }
}

/// Clamp the value to 1 if the value is over 100%.
#[inline]
fn clamp_to_one(number: NumberOrPercentage) -> NumberOrPercentage {
    match number {
        NumberOrPercentage::Percentage(mut percent) => {
            percent.clamp_to_hundred();
            NumberOrPercentage::Percentage(percent)
        },
        NumberOrPercentage::Number(mut number) => {
            number.clamp_to_one();
            NumberOrPercentage::Number(number)
        },
    }
}

type NonNegativeFactor = NonNegative<FilterFactor>;
impl NonNegativeFactor {
    fn one() -> Self {
        Self(FilterFactor(NumberOrPercentage::Number(Number::new(1.))))
    }
}

impl Parse for NonNegativeFactor {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ParseError> {
        Ok(Self(FilterFactor(
            NonNegativeNumberOrPercentage::parse(context, input)?.0,
        )))
    }
}

type ZeroToOneFactor = ZeroToOne<FilterFactor>;
impl ZeroToOneFactor {
    fn one() -> Self {
        Self(FilterFactor(NumberOrPercentage::Number(Number::new(1.))))
    }
}

impl Parse for ZeroToOneFactor {
    #[inline]
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ParseError> {
        Ok(Self(FilterFactor(clamp_to_one(
            NumberOrPercentage::parse_non_negative(context, input)?,
        ))))
    }
}

/// A specified value for the `drop-shadow()` filter.
pub type SimpleShadow = GenericSimpleShadow<Option<Color>, Length, Option<NonNegativeLength>>;

impl Parse for BoxShadow {
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ParseError> {
        let mut lengths = None;
        let mut color = None;
        let mut inset = false;

        loop {
            if !inset
                && input
                    .try_parse(|input| input.expect_ident_matching("inset"))
                    .is_ok()
            {
                inset = true;
                continue;
            }
            if lengths.is_none() {
                let value = input.try_parse::<_, _, ParseError>(|i| {
                    let horizontal = Length::parse(context, i)?;
                    let vertical = Length::parse(context, i)?;
                    let (blur, spread) =
                        match i.try_parse(|i| Length::parse_non_negative(context, i)) {
                            Ok(blur) => {
                                let spread = i.try_parse(|i| Length::parse(context, i)).ok();
                                (Some(blur.into()), spread)
                            },
                            Err(_) => (None, None),
                        };
                    Ok((horizontal, vertical, blur, spread))
                });
                if let Ok(value) = value {
                    lengths = Some(value);
                    continue;
                }
            }
            if color.is_none()
                && let Ok(value) = input.try_parse(|i| Color::parse(context, i))
            {
                color = Some(value);
                continue;
            }
            break;
        }

        let lengths = lengths.ok_or(ParseError::custom(StyleParseErrorKind::UnspecifiedError))?;
        Ok(BoxShadow {
            base: SimpleShadow {
                color,
                horizontal: lengths.0,
                vertical: lengths.1,
                blur: lengths.2,
            },
            spread: lengths.3,
            inset,
        })
    }
}

impl ToComputedValue for BoxShadow {
    type ComputedValue = ComputedBoxShadow;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        ComputedBoxShadow {
            base: self.base.to_computed_value(context),
            spread: self
                .spread
                .as_ref()
                .unwrap_or(&Length::zero())
                .to_computed_value(context),
            inset: self.inset,
        }
    }

    #[inline]
    fn from_computed_value(computed: &ComputedBoxShadow) -> Self {
        BoxShadow {
            base: ToComputedValue::from_computed_value(&computed.base),
            spread: Some(ToComputedValue::from_computed_value(&computed.spread)),
            inset: computed.inset,
        }
    }
}

// We need this for converting the specified Filter into computed Filter without Context (for
// some FFIs in glue.rs). This can fail because in some circumstances, we still need Context to
// determine the computed value.
impl Filter {
    /// Generate the ComputedFilter without Context.
    pub fn to_computed_value_without_context(&self) -> Result<ComputedFilter, ()> {
        match *self {
            Filter::Blur(ref length) => Ok(ComputedFilter::Blur(ComputedNonNegativeLength::new(
                length.0.to_computed_pixel_length_without_context()?,
            ))),
            Filter::Brightness(ref factor) => {
                Ok(ComputedFilter::Brightness(ComputedNonNegativeNumber::from(
                    factor.0.to_computed_value_without_context()?.value(),
                )))
            },
            Filter::Contrast(ref factor) => {
                Ok(ComputedFilter::Contrast(ComputedNonNegativeNumber::from(
                    factor.0.to_computed_value_without_context()?.value(),
                )))
            },
            Filter::Grayscale(ref factor) => {
                Ok(ComputedFilter::Grayscale(ComputedZeroToOneNumber::from(
                    factor.0.to_computed_value_without_context()?.value(),
                )))
            },
            Filter::HueRotate(ref angle) => Ok(ComputedFilter::HueRotate(
                ComputedAngle::from_degrees(angle.degrees().ok_or(())?),
            )),
            Filter::Invert(ref factor) => {
                Ok(ComputedFilter::Invert(ComputedZeroToOneNumber::from(
                    factor.0.to_computed_value_without_context()?.value(),
                )))
            },
            Filter::Opacity(ref factor) => {
                Ok(ComputedFilter::Opacity(ComputedZeroToOneNumber::from(
                    factor.0.to_computed_value_without_context()?.value(),
                )))
            },
            Filter::Saturate(ref factor) => {
                Ok(ComputedFilter::Saturate(ComputedNonNegativeNumber::from(
                    factor.0.to_computed_value_without_context()?.value(),
                )))
            },
            Filter::Sepia(ref factor) => Ok(ComputedFilter::Sepia(ComputedZeroToOneNumber::from(
                factor.0.to_computed_value_without_context()?.value(),
            ))),
            Filter::DropShadow(ref shadow) => {
                let color = shadow
                    .color
                    .as_ref()
                    .unwrap_or(&Color::currentcolor())
                    .to_computed_color(None)?;

                let horizontal = ComputedCSSPixelLength::new(
                    shadow
                        .horizontal
                        .to_computed_pixel_length_without_context()?,
                );
                let vertical = ComputedCSSPixelLength::new(
                    shadow.vertical.to_computed_pixel_length_without_context()?,
                );
                let blur = ComputedNonNegativeLength::new(
                    shadow
                        .blur
                        .as_ref()
                        .unwrap_or(&NonNegativeLength::zero())
                        .0
                        .to_computed_pixel_length_without_context()?,
                );

                Ok(ComputedFilter::DropShadow(ComputedSimpleShadow {
                    color,
                    horizontal,
                    vertical,
                    blur,
                }))
            },
            #[cfg(feature = "gecko")]
            Filter::Url(ref url) => Ok(ComputedFilter::Url(ComputedUrl(url.clone()))),
            // Opt-in: a servo build resolves the URL for an embedder that applies SVG filters itself.
            #[cfg(all(feature = "servo", feature = "ahcut"))]
            Filter::Url(ref url) => url
                .url()
                .cloned()
                .map(ComputedUrl::Valid)
                .map(ComputedFilter::Url)
                .ok_or(()),
            #[cfg(all(feature = "servo", not(feature = "ahcut")))]
            Filter::Url(_) => Err(()),
        }
    }
}

impl Parse for Filter {
    #[inline]
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ParseError> {
        #[cfg(any(feature = "gecko", feature = "ahcut"))]
        {
            if let Ok(url) = input.try_parse(|i| SpecifiedUrl::parse(context, i)) {
                return Ok(GenericFilter::Url(url));
            }
        }
        let function = {
            let f = input.expect_function()?;
            f.clone()
        };
        input.parse_nested_block(|i| {
            match_ignore_ascii_case! { &*function,
                "blur" => Ok(GenericFilter::Blur(
                    i.try_parse(|i| NonNegativeLength::parse(context, i))
                     .unwrap_or(Zero::zero()),
                )),
                "brightness" => Ok(GenericFilter::Brightness(
                    i.try_parse(|i| NonNegativeFactor::parse(context, i))
                     .unwrap_or(NonNegativeFactor::one()),
                )),
                "contrast" => Ok(GenericFilter::Contrast(
                    i.try_parse(|i| NonNegativeFactor::parse(context, i))
                     .unwrap_or(NonNegativeFactor::one()),
                )),
                "grayscale" => {
                    // Values of amount over 100% are allowed but UAs must clamp the values to 1.
                    // https://drafts.fxtf.org/filter-effects/#funcdef-filter-grayscale
                    Ok(GenericFilter::Grayscale(
                        i.try_parse(|i| ZeroToOneFactor::parse(context, i))
                         .unwrap_or(ZeroToOneFactor::one()),
                    ))
                },
                "hue-rotate" => {
                    // We allow unitless zero here, see:
                    // https://github.com/w3c/fxtf-drafts/issues/228
                    Ok(GenericFilter::HueRotate(
                        i.try_parse(|i| Angle::parse_with_unitless(context, i))
                         .unwrap_or(Zero::zero()),
                    ))
                },
                "invert" => {
                    // Values of amount over 100% are allowed but UAs must clamp the values to 1.
                    // https://drafts.fxtf.org/filter-effects/#funcdef-filter-invert
                    Ok(GenericFilter::Invert(
                        i.try_parse(|i| ZeroToOneFactor::parse(context, i))
                         .unwrap_or(ZeroToOneFactor::one()),
                    ))
                },
                "opacity" => {
                    // Values of amount over 100% are allowed but UAs must clamp the values to 1.
                    // https://drafts.fxtf.org/filter-effects/#funcdef-filter-opacity
                    Ok(GenericFilter::Opacity(
                        i.try_parse(|i| ZeroToOneFactor::parse(context, i))
                         .unwrap_or(ZeroToOneFactor::one()),
                    ))
                },
                "saturate" => Ok(GenericFilter::Saturate(
                    i.try_parse(|i| NonNegativeFactor::parse(context, i))
                     .unwrap_or(NonNegativeFactor::one()),
                )),
                "sepia" => {
                    // Values of amount over 100% are allowed but UAs must clamp the values to 1.
                    // https://drafts.fxtf.org/filter-effects/#funcdef-filter-sepia
                    Ok(GenericFilter::Sepia(
                        i.try_parse(|i| ZeroToOneFactor::parse(context, i))
                         .unwrap_or(ZeroToOneFactor::one()),
                    ))
                },
                "drop-shadow" => Ok(GenericFilter::DropShadow(Parse::parse(context, i)?)),
                _ => Err(ParseError::custom(StyleParseErrorKind::UnspecifiedError)),
            }
        })
    }
}

impl Parse for SimpleShadow {
    #[inline]
    fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ParseError> {
        let color = input.try_parse(|i| Color::parse(context, i)).ok();
        let horizontal = Length::parse(context, input)?;
        let vertical = Length::parse(context, input)?;
        let blur = input
            .try_parse(|i| Length::parse_non_negative(context, i))
            .ok();
        let blur = blur.map(NonNegative::<Length>);
        let color = color.or_else(|| input.try_parse(|i| Color::parse(context, i)).ok());

        Ok(SimpleShadow {
            color,
            horizontal,
            vertical,
            blur,
        })
    }
}

impl ToComputedValue for SimpleShadow {
    type ComputedValue = ComputedSimpleShadow;

    #[inline]
    fn to_computed_value(&self, context: &Context) -> Self::ComputedValue {
        ComputedSimpleShadow {
            color: self
                .color
                .as_ref()
                .unwrap_or(&Color::currentcolor())
                .to_computed_value(context),
            horizontal: self.horizontal.to_computed_value(context),
            vertical: self.vertical.to_computed_value(context),
            blur: self
                .blur
                .as_ref()
                .unwrap_or(&NonNegativeLength::zero())
                .to_computed_value(context),
        }
    }

    #[inline]
    fn from_computed_value(computed: &Self::ComputedValue) -> Self {
        SimpleShadow {
            color: Some(ToComputedValue::from_computed_value(&computed.color)),
            horizontal: ToComputedValue::from_computed_value(&computed.horizontal),
            vertical: ToComputedValue::from_computed_value(&computed.vertical),
            blur: Some(ToComputedValue::from_computed_value(&computed.blur)),
        }
    }
}

/// https://drafts.fxtf.org/compositing/#propdef-mix-blend-mode
#[allow(missing_docs)]
#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    Eq,
    FromPrimitive,
    Hash,
    MallocSizeOf,
    Parse,
    PartialEq,
    Serialize,
    SpecifiedValueInfo,
    ToComputedValue,
    ToCss,
    ToResolvedValue,
    ToShmem,
    ToTyped,
)]
#[repr(u8)]
pub enum Blend {
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,
    PlusLighter,
}

#[cfg(all(test, feature = "servo", feature = "ahcut"))]
mod servo_filter_url_tests {
    use super::*;
    use crate::context::QuirksMode;
    use crate::custom_properties::AttrTaint;
    use crate::stylesheets::{Namespaces, Origin, UrlExtraData};
    use crate::values::animated::{ToAnimatedValue, effects::AnimatedFilter};
    use std::borrow::Cow;
    use style_traits::ParsingMode;

    fn parse_filter(css: &str, base: &str) -> Result<Filter, ParseError> {
        let url_data = UrlExtraData::from(url::Url::parse(base).unwrap());
        let context = ParserContext::new(
            Origin::Author,
            &url_data,
            None,
            ParsingMode::DEFAULT,
            QuirksMode::NoQuirks,
            Cow::Owned(Namespaces::default()),
            None,
            None,
            AttrTaint::default(),
        );
        Parser::new(css).parse_entirely(|input| Filter::parse(&context, input))
    }

    #[test]
    fn local_urls_use_real_css_decoding_and_computed_roundtrip() {
        for css in ["url(#mb)", r##"url("#mb")"##, r"u\72l(#mb)"] {
            let specified = parse_filter(css, "https://example.invalid/document.html").unwrap();
            let computed = specified.to_computed_value_without_context().unwrap();
            let ComputedFilter::Url(reference) = &computed else {
                panic!("URL syntax lost at computation: {css}");
            };
            assert_eq!(
                reference.url().unwrap().as_str(),
                "https://example.invalid/document.html#mb"
            );
            assert_eq!(Filter::from_computed_value(&computed), specified);
        }
    }

    #[test]
    fn urls_retain_stylesheet_base_for_the_renderer_to_check() {
        let computed = parse_filter("url(#mb)", "https://example.invalid/styles/linked.css")
            .unwrap()
            .to_computed_value_without_context()
            .unwrap();
        let ComputedFilter::Url(reference) = computed else {
            panic!("URL did not survive computation");
        };
        // Lexing a CSS URL is not permission to fetch or paint an external filter.
        // The unchanged Blitz provenance/local-document resolver must refuse it.
        assert_eq!(
            reference.url().unwrap().as_str(),
            "https://example.invalid/styles/linked.css#mb"
        );
    }

    #[test]
    fn malformed_nested_and_trailing_tokens_are_not_urls() {
        for css in [
            "url(#mb trailing)",
            "url(var(--filter))",
            "url(\"#mb\") trailing",
            "unknown(#mb)",
            "url(\"bad\nstring\")",
        ] {
            assert!(
                parse_filter(css, "https://example.invalid/document.html").is_err(),
                "{css}"
            );
        }
    }

    #[test]
    fn invalid_url_has_no_contextless_valid_reference() {
        let specified = parse_filter("url(\"\")", "https://example.invalid/document.html").unwrap();
        assert!(matches!(specified, Filter::Url(_)));
        assert!(specified.to_computed_value_without_context().is_err());
    }

    #[test]
    fn specified_computed_animated_aliases_share_the_real_url_type() {
        fn specified_bridge<T: ToComputedValue<ComputedValue = ComputedFilter>>() {}
        fn animated_bridge<T: ToAnimatedValue<AnimatedValue = AnimatedFilter>>() {}
        specified_bridge::<Filter>();
        animated_bridge::<ComputedFilter>();
    }

    #[test]
    fn ordinary_blur_keeps_its_existing_computation() {
        let computed = parse_filter("blur(12px)", "https://example.invalid/document.html")
            .unwrap()
            .to_computed_value_without_context()
            .unwrap();
        let ComputedFilter::Blur(radius) = computed else {
            panic!("blur variant changed");
        };
        assert_eq!(radius.px(), 12.0);
    }
}
