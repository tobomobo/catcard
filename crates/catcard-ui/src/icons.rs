//! How a screen names the confirm and cancel keys.
//!
//! The `y` and `x` this code calls them are names from a wiring diagram, not from the
//! device. What a person is looking at differs by board: the mk pads carry a **✓** and a
//! **✗** moulded into the caps, and the Q1's keys are printed ENTER and CANCEL. Either
//! way a screen should show what is under their thumb -- a hint that says something else
//! asks them to translate, which is the thing these marks exist to avoid.
//!
//! So a board picks its [`KeyMark`]s and every hint line draws whichever it has. The
//! symbols are bitmaps rather than font glyphs because both bitmap faces here are ASCII,
//! generated from BDF, and `✓` is U+2713: adding codepoints to a generated face to spell
//! two symbols would be a worse trade than five bytes of pixels each.

use crate::art::Bitmap;

/// ✓ — the key this code calls [`crate::keypad::Key::Confirm`].
pub const CHECK: Bitmap = Bitmap {
    width: 5,
    height: 5,
    bytes_per_row: 1,
    // ....#
    // ...#.
    // #..#.
    // .#.#.
    // ..#..
    pixels: &[0x08, 0x10, 0x90, 0x50, 0x20],
};

/// ✗ — the key this code calls [`crate::keypad::Key::Cancel`].
pub const CROSS: Bitmap = Bitmap {
    width: 5,
    height: 5,
    bytes_per_row: 1,
    // #...#
    // .#.#.
    // ..#..
    // .#.#.
    // #...#
    pixels: &[0x88, 0x50, 0x20, 0x50, 0x88],
};

/// A folder, for a directory row in the file browser. 7x7.
pub const FOLDER: Bitmap = Bitmap {
    width: 7,
    height: 7,
    bytes_per_row: 1,
    // ##.....
    // ######.
    // #.....#
    // #.....#
    // #.....#
    // #######
    // .......
    pixels: &[0xC0, 0xFC, 0x82, 0x82, 0x82, 0xFE, 0x00],
};

/// A document, for a file row in the file browser. 7x7.
pub const FILE: Bitmap = Bitmap {
    width: 7,
    height: 7,
    bytes_per_row: 1,
    // #####..
    // #...##.
    // #....#.
    // #....#.
    // #....#.
    // #....#.
    // ######.
    pixels: &[0xF8, 0x8C, 0x84, 0x84, 0x84, 0x84, 0xFC],
};

/// A left arrow, for the "parent directory" row. 7x7.
pub const BACK: Bitmap = Bitmap {
    width: 7,
    height: 7,
    bytes_per_row: 1,
    // ...#...
    // ..##...
    // .######
    // #######
    // .######
    // ..##...
    // ...#...
    pixels: &[0x10, 0x30, 0x7E, 0xFE, 0x7E, 0x30, 0x10],
};

/// What a board's confirm or cancel key is labelled with.
///
/// A fact about the hardware, like the key map: the board layer picks these, not the
/// layout, because it is about the caps rather than the panel.
#[derive(Copy, Clone)]
pub enum KeyMark {
    /// A symbol moulded into the cap, drawn as pixels beside the label.
    Icon(&'static Bitmap),
    /// A word printed on the cap, drawn in the same face as the label.
    Word(&'static str),
}

/// Draw `<mark> label`, returning where the text ended.
pub fn draw_key_hint<C: crate::canvas::Canvas + ?Sized, F: crate::face::Face + ?Sized>(
    fb: &mut C,
    font: &F,
    x: usize,
    y: usize,
    mark: KeyMark,
    label: &str,
) -> usize {
    match mark {
        KeyMark::Icon(icon) => draw_hint(fb, icon, font, x, y, label),
        KeyMark::Word(word) => {
            let at = crate::text::draw_text(fb, font, x, y, word);
            crate::text::draw_text(fb, font, at + font.advance(b' '), y, label)
        }
    }
}

/// Width [`draw_key_hint`] will occupy, for centring a line before drawing it.
pub fn key_hint_width<F: crate::face::Face + ?Sized>(
    font: &F,
    mark: KeyMark,
    label: &str,
) -> usize {
    match mark {
        KeyMark::Icon(_) => hint_width(font, label),
        KeyMark::Word(word) => {
            crate::text::width_of(font, word)
                + font.advance(b' ')
                + crate::text::width_of(font, label)
        }
    }
}

/// A question's two answers on one centred line, `<yes mark> yes   <no mark> no`, each key
/// shown as the board marks it ([`KeyMark`]): the moulded tick and cross on the numpad
/// boards, the printed words on the Q1. `y` is the top of the line.
pub fn draw_answer_row<C: crate::canvas::Canvas + ?Sized, F: crate::face::Face + ?Sized>(
    fb: &mut C,
    font: &F,
    y: usize,
    yes: (KeyMark, &str),
    no: (KeyMark, &str),
) {
    let gap = 3 * font.advance(b' ');
    let total = key_hint_width(font, yes.0, yes.1) + gap + key_hint_width(font, no.0, no.1);
    let x = fb.width().saturating_sub(total) / 2;
    let x = draw_key_hint(fb, font, x, y, yes.0, yes.1) + gap;
    draw_key_hint(fb, font, x, y, no.0, no.1);
}

/// Width of an icon plus the gap before the word after it, at 1x.
pub const ICON_ADVANCE: usize = 7;

/// How much to grow a 5x5 icon so it reads beside `font`.
///
/// The icons are drawn as pixels, not as glyphs, so they do not grow with the face on
/// their own: beside the 4x6 status face they are the size they were drawn, and beside a
/// 7x14 one they need doubling. One step per seven pixels of line height, never below 1.
pub fn icon_scale<F: crate::face::Face + ?Sized>(font: &F) -> usize {
    (font.line_height() / 7).max(1)
}

/// Draw `✓ label` or `✗ label`, returning where the text ended.
///
/// The icon is centred against the font's cell rather than sharing its baseline, because
/// a 5-pixel square hung off a 14-pixel face's baseline sits visibly low.
pub fn draw_hint<C: crate::canvas::Canvas + ?Sized, F: crate::face::Face + ?Sized>(
    fb: &mut C,
    icon: &Bitmap,
    font: &F,
    x: usize,
    y: usize,
    label: &str,
) -> usize {
    let scale = icon_scale(font);
    let drop = font
        .line_height()
        .saturating_sub(icon.height as usize * scale)
        / 2;
    crate::splash::draw_bitmap_scaled(fb, icon, x, y + drop, scale);
    crate::text::draw_text(fb, font, x + ICON_ADVANCE * scale, y, label)
}

/// Width `draw_hint` will occupy, for centring a line before drawing it.
pub fn hint_width<F: crate::face::Face + ?Sized>(font: &F, label: &str) -> usize {
    ICON_ADVANCE * icon_scale(font) + crate::text::width_of(font, label)
}

/// Radius of a typed place's dot in a [`draw_dot_row`], at scale 1.
pub const DOT_R: usize = 3;
/// Radius of a place still to fill.
const SLOT_R: usize = 1;
/// From one place's centre to the next, at scale 1.
const DOT_STEP: usize = 10;

/// The box a row of `max` places occupies at `scale`: (width, height).
///
/// Sized for the most places the field holds, not for how many are filled, so the caller
/// places the row once and the dots fill it from the left: nothing already drawn moves
/// when a digit goes in.
pub const fn dot_row_size(max: usize, scale: usize) -> (usize, usize) {
    let d = 2 * DOT_R + 1;
    let w = if max == 0 {
        0
    } else {
        (max - 1) * DOT_STEP + d
    };
    (w * scale, d * scale)
}

/// A row of `max` places from `(x, y)`, the top-left of its [`dot_row_size`] box: a filled
/// dot in `ink` for each of the first `typed`, and a small one in `slot`, where given, for
/// each still to come.
///
/// One dot per digit, the same as a `*` gave away -- how many, and nothing about which.
/// The small dots say how many the row holds before anything is typed.
#[allow(clippy::too_many_arguments)]
pub fn draw_dot_row<C: crate::canvas::Canvas + ?Sized>(
    fb: &mut C,
    typed: usize,
    max: usize,
    x: usize,
    y: usize,
    scale: usize,
    ink: crate::canvas::Level,
    slot: Option<crate::canvas::Level>,
) {
    let scale = scale.max(1);
    let cy = y + DOT_R * scale;
    for i in 0..max {
        let cx = x + (i * DOT_STEP + DOT_R) * scale;
        if i < typed {
            disc(fb, cx, cy, DOT_R * scale, ink);
        } else if let Some(slot) = slot {
            disc(fb, cx, cy, SLOT_R * scale, slot);
        }
    }
}

/// A filled circle of radius `r` centred on `(cx, cy)`.
pub fn disc<C: crate::canvas::Canvas + ?Sized>(
    c: &mut C,
    cx: usize,
    cy: usize,
    r: usize,
    ink: crate::canvas::Level,
) {
    let r = r as isize;
    // A half-pixel's grace on the edge, so small circles come out round, not diamond.
    let limit = r * r + r;
    for dy in -r..=r {
        let mut half = 0;
        while (half + 1) * (half + 1) + dy * dy <= limit {
            half += 1;
        }
        let y = cy as isize + dy;
        let x = cx as isize - half;
        if y >= 0 && x >= 0 {
            c.fill_rect(x as usize, y as usize, (2 * half + 1) as usize, 1, ink);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::{Canvas, Gray320x240, PAPER};
    use crate::font::{misc4x6, peep7x14};

    #[test]
    fn a_word_mark_reads_as_the_cap_does_and_measures_the_same() {
        // The Q1's keys say ENTER and CANCEL; drawing a tick there would name a key the
        // board does not have.
        let f = &peep7x14::FONT;
        let mark = KeyMark::Word("ENTER");
        let mut c = Gray320x240::new();
        let end = draw_key_hint(&mut c, f, 0, 0, mark, "accept");
        assert_eq!(end, ("ENTER".len() + 1 + "accept".len()) * 7);
        assert_eq!(
            key_hint_width(f, mark, "accept"),
            end,
            "width disagrees with the draw"
        );
        assert!(
            (0..14).any(|y| (0..35).any(|x| c.get(x, y) != PAPER)),
            "no word drawn"
        );
    }

    #[test]
    fn an_icon_mark_measures_the_same_as_the_icon_hint() {
        let f = &peep7x14::FONT;
        let mark = KeyMark::Icon(&CHECK);
        assert_eq!(key_hint_width(f, mark, "yes"), hint_width(f, "yes"));
        let mut a = Gray320x240::new();
        let mut b = Gray320x240::new();
        let ea = draw_key_hint(&mut a, f, 0, 0, mark, "yes");
        let eb = draw_hint(&mut b, &CHECK, f, 0, 0, "yes");
        assert_eq!(ea, eb);
    }

    #[test]
    fn an_icon_grows_with_the_face_beside_it_and_the_label_follows() {
        assert_eq!(icon_scale(&misc4x6::FONT), 1, "4x6 keeps the drawn size");
        assert_eq!(icon_scale(&peep7x14::FONT), 2, "7x14 doubles it");

        // The label starts past the scaled icon, and the reported width covers both.
        let mut c = Gray320x240::new();
        let end = draw_hint(&mut c, &CHECK, &peep7x14::FONT, 0, 0, "yes");
        assert_eq!(end, ICON_ADVANCE * 2 + 3 * 7);
        assert_eq!(hint_width(&peep7x14::FONT, "yes"), end);
        // A 5x5 tick at 2x covers ten rows; at 1x it would stop at five.
        let ink_rows = (0..20)
            .filter(|&y| (0..ICON_ADVANCE * 2).any(|x| c.get(x, y) != PAPER))
            .count();
        assert!(
            ink_rows >= 9,
            "the tick did not grow: {ink_rows} rows of ink"
        );
    }

    /// Both symbols must actually have pixels in them, and be square.
    ///
    /// A bitmap whose bytes are all zero draws nothing at all, and "nothing at all" on a
    /// key hint is indistinguishable from a label that was never written.
    #[test]
    fn both_icons_carry_ink_and_are_the_size_they_claim() {
        for (name, b) in [("check", &CHECK), ("cross", &CROSS)] {
            assert_eq!(b.pixels.len(), b.height as usize, "{name}: wrong row count");
            let ink = b.pixels.iter().filter(|p| **p != 0).count();
            assert_eq!(ink, b.height as usize, "{name}: every row should have ink");
            // Nothing may sit in the padding bits, or it draws outside the cell.
            for (y, row) in b.pixels.iter().enumerate() {
                assert_eq!(
                    row & ((1 << (8 - b.width)) - 1),
                    0,
                    "{name}: row {y} has ink in the padding bits"
                );
            }
        }
    }

    /// The cross is symmetric; the check is not. This is how a flipped or mirrored
    /// bitmap gets caught, which matters because both are drawn at 5x5 where a mistake
    /// reads as "slightly odd" rather than as obviously wrong.
    #[test]
    fn the_cross_is_symmetric_and_the_check_is_not() {
        let mirror = |row: u8, w: u16| {
            let mut out = 0u8;
            for x in 0..w {
                if row & (0x80 >> x) != 0 {
                    out |= 0x80 >> (w - 1 - x);
                }
            }
            out
        };
        for (y, row) in CROSS.pixels.iter().enumerate() {
            assert_eq!(*row, mirror(*row, CROSS.width), "cross row {y} is lopsided");
        }
        assert_eq!(
            CROSS.pixels[0], CROSS.pixels[4],
            "cross is not top-bottom even"
        );
        assert!(
            CHECK
                .pixels
                .iter()
                .enumerate()
                .any(|(y, r)| *r != mirror(*r, CHECK.width)
                    || CHECK.pixels[y] != CHECK.pixels[CHECK.height as usize - 1 - y]),
            "the check should not be symmetric -- it would not read as a tick"
        );
    }

    /// The tick's vertex is its lowest point, with both arms above it.
    #[test]
    fn the_check_has_a_vertex_at_the_bottom() {
        let lowest = CHECK.pixels.len() - 1;
        assert_eq!(
            CHECK.pixels[lowest].count_ones(),
            1,
            "the bottom row should be the single vertex pixel"
        );
        assert!(
            CHECK.pixels[0].count_ones() == 1,
            "the top row should be the tip of the long arm"
        );
    }

    use crate::canvas::INK;
    use crate::framebuffer::Mono128x64;

    /// The inked cells of a canvas, as (x, y) pairs.
    fn inked<C: Canvas>(c: &C) -> Vec<(usize, usize)> {
        let mut v = Vec::new();
        for y in 0..c.height() {
            for x in 0..c.width() {
                if c.get(x, y) == INK {
                    v.push((x, y));
                }
            }
        }
        v
    }

    #[test]
    fn the_longest_pin_part_fits_both_panels() {
        // Six digits is the most a PIN part can have; its row must fit the mono panel at
        // 1x and the Q1 at 3x.
        let (w, h) = dot_row_size(6, 1);
        assert!(w <= 128 && h <= 22, "mono: {w}x{h}");
        let (w, h) = dot_row_size(6, 3);
        assert!(w <= 320 && h <= 82, "q1: {w}x{h}");
    }

    #[test]
    fn typed_places_are_dots_and_the_rest_are_smaller() {
        let mut c = Mono128x64::new();
        draw_dot_row(&mut c, 2, 6, 10, 20, 1, INK, Some(INK));
        let lit = inked(&c);
        let in_place = |i: usize| {
            let x0 = 10 + i * DOT_STEP;
            lit.iter()
                .filter(|&&(x, _)| (x0..x0 + 2 * DOT_R + 1).contains(&x))
                .count()
        };
        let (dot, slot) = (in_place(0), in_place(5));
        assert_eq!(in_place(1), dot, "two typed dots alike");
        assert!(slot > 0, "an empty place shows");
        assert!(
            dot >= 4 * slot,
            "a typed dot ({dot}) stands out from a slot ({slot})"
        );
        // Nothing outside the row's box.
        let (w, h) = dot_row_size(6, 1);
        assert!(
            lit.iter()
                .all(|&(x, y)| (10..10 + w).contains(&x) && (20..20 + h).contains(&y))
        );
    }

    #[test]
    fn each_digit_adds_one_dot_and_nothing_moves() {
        // The row is anchored once, so typing fills the next place and leaves the earlier
        // ones exactly where they were.
        let mut before = Gray320x240::new();
        draw_dot_row(&mut before, 3, 6, 40, 90, 3, INK, None);
        let mut after = Gray320x240::new();
        draw_dot_row(&mut after, 4, 6, 40, 90, 3, INK, None);
        let b = inked(&before);
        let a = inked(&after);
        assert!(b.iter().all(|p| a.contains(p)), "an earlier dot moved");
        assert!(a.len() > b.len());
    }
}
