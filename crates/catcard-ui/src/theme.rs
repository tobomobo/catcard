//! What a level means on a page, whichever palette the page goes out through.
//!
//! A canvas holds a level per pixel and the panel's palette decides its colour, so "draw
//! this label in grey" is only a sentence about a level once every palette a page can be
//! shown through agrees what that level looks like. The Q1 has two for ordinary pages:
//! [`LIST`], the grey ramp lists and documents paint in, and [`PAGE`], the icons' palette
//! that the typing screens and approvals paint in so they can carry art. Their levels
//! 10 to 13 are the same four colours here, and those are the **text roles**:
//!
//! | level | role | looks like |
//! |---|---|---|
//! | [`INK`] | what the page is about: titles, values, typing | white |
//! | [`SECONDARY`] | labels, captions, hints | mid grey |
//! | [`RULE`] | lines that divide a page | dark grey |
//! | [`SLOT`] | an empty place on a white card | light grey |
//! | [`ACCENT`] | what must not be missed | orange |
//!
//! All of them are 8 or above, so on the one-bit OLED -- which lights a pixel at 8 and
//! up -- every role still draws, as white. The hierarchy there comes from position and
//! face, as it always has; on the Q1 it also comes from colour.
//!
//! Hierarchy is the reason these exist. Before them every word on a Q1 page was the same
//! white, so a heading, a label, a value and a hint could only be told apart by where
//! they sat -- and on a typing screen a field's label sat inside the field, in the same
//! ink as what was typed.

pub use crate::canvas::INK;
use crate::canvas::Level;
use crate::st7789::{SLATE, rgb565};

/// Labels, captions and hints: present, readable, quieter than the content.
pub const SECONDARY: Level = 10;
/// A line that divides a page: under a title, between parts.
pub const RULE: Level = 11;
/// A place on a white card that has not been filled yet: an empty PIN digit.
pub const SLOT: Level = 12;
/// The one thing on a page that must not be missed: tries running out.
pub const ACCENT: Level = 13;

/// `#A0A0A0`: about 5:1 against the `#303030` page, comfortably readable as small text.
const SECONDARY_RGB: u16 = rgb565(0xA0 >> 3, 0xA0 >> 2, 0xA0 >> 3);
/// `#585858`: visible as a line, never mistaken for text.
const RULE_RGB: u16 = rgb565(0x58 >> 3, 0x58 >> 2, 0x58 >> 3);
/// `#C8C8C8`: on white, a place rather than a mark.
const SLOT_RGB: u16 = rgb565(0xC8 >> 3, 0xC8 >> 2, 0xC8 >> 3);
/// The cats' orange, the same entry the menu icons use (`menuicons::PALETTE[7]`).
const ACCENT_RGB: u16 = 0xF465;

/// Lay the roles over `base`'s levels 10 to 13.
const fn with_roles(base: [u16; 16]) -> [u16; 16] {
    let mut p = base;
    p[SECONDARY as usize] = SECONDARY_RGB;
    p[RULE as usize] = RULE_RGB;
    p[SLOT as usize] = SLOT_RGB;
    p[ACCENT as usize] = ACCENT_RGB;
    p
}

/// Lists and documents: [`SLATE`] with the roles in place of four of its steps.
///
/// Nothing drew those steps as a ramp -- the faces are one bit, so text never lands
/// between ink and paper -- and the levels screens did use as greys (the scrollbar's
/// track at 4, the help strip's label at 9) are below them and unchanged.
pub const LIST: [u16; 16] = with_roles(SLATE);

/// Typing screens, approvals and the icon grid: the icons' palette with the roles in its
/// unused entries. The art draws in 1 to 9 and 15; 10 to 14 were black and unused.
pub const PAGE: [u16; 16] = with_roles(crate::art::menuicons::PALETTE);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::art::menuicons::PALETTE;

    /// A role means one colour, whichever page it is drawn on.
    #[test]
    fn both_palettes_agree_on_every_role() {
        for role in [SECONDARY, RULE, SLOT, ACCENT, INK] {
            assert_eq!(LIST[role as usize], PAGE[role as usize], "level {role}");
        }
        // And the background is the same grey, so moving between them is not a flash.
        assert_eq!(LIST[0], PAGE[0]);
    }

    /// The roles took only levels the art leaves free.
    #[test]
    fn the_art_keeps_its_colours() {
        for i in (0..10).chain([15]) {
            assert_eq!(PAGE[i], PALETTE[i], "art level {i}");
        }
        for (i, &c) in PALETTE.iter().enumerate().take(15).skip(10) {
            assert_eq!(c, 0, "level {i} was in use by the art");
        }
    }

    /// Every role lights on the OLED, which lights a level of 8 or more.
    #[test]
    fn every_role_draws_on_one_bit() {
        for role in [SECONDARY, RULE, SLOT, ACCENT] {
            assert!(role >= 8);
        }
    }
}
