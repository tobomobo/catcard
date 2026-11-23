//! A question that needs a yes, laid out as a page with a picture: the colour panel's
//! version of "allow this?".
//!
//! The action heads the page, a picture sits under it, and what the question is about --
//! a site, a file, a device -- is the line under the picture, in the body face and wrapped
//! if it is long. Smaller facts follow (the account, which wallet), and the keys that
//! answer it sit along the bottom. Nothing scrolls: what does not fit is left out rather
//! than pushed below the keys, so the answer is always on the screen.
//!
//! Drawn in the icon palette (`art::menuicons::PALETTE`), where text levels and the art's
//! colours share one table, the way the picture pages already draw.

use crate::art::indexed::{Indexed, draw_indexed};
use crate::canvas::{Canvas, INK, Level, PAPER};
use crate::face::Face;
use crate::icons::{KeyMark, draw_answer_row};
use crate::scroll::Fonts;
use crate::text::{centred, draw_text_in, width_of, wrap};
use crate::theme::{RULE, SECONDARY};

/// What the page says.
pub struct Approval<'a> {
    /// The action, as the heading: "Register", "Sign in".
    pub head: &'a str,
    /// The picture under it, on a panel that can show one.
    pub art: Option<&'a Indexed>,
    /// What the question is about, the page's main line: at most two lines when wrapped,
    /// or three in the small face where the body face would need more.
    pub main: &'a str,
    /// Smaller lines under it, each wrapped to at most two lines, dropped from the end
    /// when they would reach the keys.
    pub small: &'a [&'a str],
    /// The key that says yes, as the board marks it, and what it does: `allow`.
    pub yes: (KeyMark, &'a str),
    /// The key that says no, and what it does: `refuse`.
    pub no: (KeyMark, &'a str),
}

/// Draw `text` centred in `face` from `y`, at most `most` lines (up to two), or in
/// `fallback` -- up to three -- when `face` would need more. Stops above `limit`. Returns where the next line goes.
#[allow(clippy::too_many_arguments)]
fn block<C: Canvas + ?Sized>(
    c: &mut C,
    face: &dyn Face,
    most: usize,
    fallback: &dyn Face,
    fonts: &Fonts<'_>,
    text: &str,
    mut y: usize,
    limit: usize,
    ink: Level,
) -> usize {
    let (lines, f) = set(c.width(), face, most, fallback, fonts, text);
    for l in &lines {
        if y + f.line_height() > limit {
            break;
        }
        draw_text_in(c, f, centred(f, l, c.width()), y, l, ink);
        y += f.line_height() + fonts.gap;
    }
    y
}

/// How [`block`] sets `text` on a canvas `w` wide: its lines, and the face they are in.
fn set<'t, 'f>(
    w: usize,
    face: &'f dyn Face,
    most: usize,
    fallback: &'f dyn Face,
    fonts: &Fonts<'_>,
    text: &'t str,
) -> (heapless::Vec<&'t str, 3>, &'f dyn Face) {
    let width = w.saturating_sub(2 * fonts.margin);
    let (two, cut) = wrap::<_, 2>(face, text, width);
    if cut || two.len() > most {
        (wrap::<_, 3>(fallback, text, width).0, fallback)
    } else {
        (two.iter().copied().collect(), face)
    }
}

/// How far [`block`] moves down for `text`, with room for every line.
fn block_height(
    w: usize,
    face: &dyn Face,
    most: usize,
    fallback: &dyn Face,
    fonts: &Fonts<'_>,
    text: &str,
) -> usize {
    let (lines, f) = set(w, face, most, fallback, fonts, text);
    lines.len() * (f.line_height() + fonts.gap)
}

/// Draw `a` over the whole canvas.
pub fn draw<C: Canvas + ?Sized>(c: &mut C, fonts: &Fonts<'_>, a: &Approval<'_>) {
    c.clear();
    let (w, h) = (c.width(), c.height());
    // Spacing follows the panel's own margin: generous on the colour panel, a couple of
    // pixels on the 64-row one.
    let space = fonts.margin + 2;

    // The answers own the bottom line; everything else stops short of it.
    let keys_y = h.saturating_sub(keys_height(fonts, a));
    let limit = keys_y.saturating_sub(space / 2);

    // The heading gets one line in its own face: on the 64-row panel a second would take
    // the room the warning under it needs.
    let mut y = block(
        c,
        fonts.title,
        1,
        fonts.small,
        fonts,
        a.head,
        space,
        limit,
        INK,
    );
    if fonts.title_rule {
        // The rule every page on the panel has under its title.
        c.fill_rect(fonts.margin, y, w.saturating_sub(2 * fonts.margin), 1, RULE);
        y += 1;
    }
    y += space;
    // The words win over the picture: it is drawn only if all of them still fit under it.
    // A picture is the page's look; the small print is what the owner is agreeing to.
    let words = {
        let main = if a.main.is_empty() {
            0
        } else {
            block_height(w, fonts.body, 2, fonts.small, fonts, a.main) + space / 2
        };
        let small: usize = a
            .small
            .iter()
            .map(|s| block_height(w, fonts.small, 2, fonts.small, fonts, s))
            .sum();
        main + small
    };
    if let Some(art) = a.art {
        let (aw, ah) = (art.width as usize, art.height as usize);
        if y + ah + space + words <= limit {
            draw_indexed(c, art, w.saturating_sub(aw) / 2, y);
            y += ah + space;
        }
    }
    if !a.main.is_empty() {
        y = block(c, fonts.body, 2, fonts.small, fonts, a.main, y, limit, INK) + space / 2;
    }
    // The small print is the page's footnotes: quieter than what is being asked.
    for s in a.small {
        y = block(
            c,
            fonts.small,
            2,
            fonts.small,
            fonts,
            s,
            y,
            limit,
            SECONDARY,
        );
    }

    draw_answers(c, fonts, keys_y, a.yes, a.no);
}

/// How many rows [`draw_answers`] takes at the foot of a page.
pub fn answers_height(fonts: &Fonts<'_>, yes: KeyMark, no: KeyMark) -> usize {
    let lh = fonts.small.line_height();
    if caps(fonts, yes, no) {
        1 + CAP_AIR + lh + 2 * CAP_Y + CAP_AIR
    } else {
        lh + (fonts.margin + 2) / 2
    }
}

/// A page's two answers along its foot, from `top`: as key caps under a rule where the
/// keys are words and the panel has colour, or as the plain hint row where they are not.
/// Any page whose answer is a key press says it this way, so a question looks like one
/// wherever it is asked.
pub fn draw_answers<C: Canvas + ?Sized>(
    c: &mut C,
    fonts: &Fonts<'_>,
    top: usize,
    yes: (KeyMark, &str),
    no: (KeyMark, &str),
) {
    match (yes.0, no.0) {
        (KeyMark::Word(y), KeyMark::Word(n)) if fonts.colour => {
            c.fill_rect(0, top, c.width(), 1, RULE);
            answer_caps(c, fonts.small, top + 1 + CAP_AIR, (y, yes.1), (n, no.1));
        }
        _ => draw_answer_row(c, fonts.small, top, yes, no),
    }
}

/// Room around a key cap's word, across and down.
const CAP_X: usize = 6;
const CAP_Y: usize = 2;
/// Above and below the row of caps.
const CAP_AIR: usize = 5;

/// A key's printed word and what pressing it does.
type Answer<'a> = (&'static str, &'a str);

/// Whether the answers are drawn as key caps: the keys are words and the panel has colour.
fn caps(fonts: &Fonts<'_>, yes: KeyMark, no: KeyMark) -> bool {
    fonts.colour && matches!((yes, no), (KeyMark::Word(_), KeyMark::Word(_)))
}

/// How many rows the answers take at the foot of the page.
fn keys_height(fonts: &Fonts<'_>, a: &Approval<'_>) -> usize {
    answers_height(fonts, a.yes.0, a.no.0)
}

/// `[ENTER] share     [CANCEL] refuse`, centred: each key drawn as a cap with its printed
/// word in it, and what it does beside it. Yes is a filled cap and no an outline, so the
/// two answers differ at a glance and the one that acts is the one that stands out.
fn answer_caps<C: Canvas + ?Sized>(
    c: &mut C,
    font: &dyn Face,
    y: usize,
    yes: Answer<'_>,
    no: Answer<'_>,
) {
    let space = font.advance(b' ');
    let cap_w = |key: &str| width_of(font, key) + 2 * CAP_X;
    let one = |(key, does): Answer<'_>| cap_w(key) + space + width_of(font, does);
    let gap = 4 * space;
    let mut x = c.width().saturating_sub(one(yes) + gap + one(no)) / 2;
    let kh = font.line_height() + 2 * CAP_Y;
    for ((key, does), filled) in [(yes, true), (no, false)] {
        let kw = cap_w(key);
        let word_ink = if filled {
            c.fill_rect(x, y, kw, kh, INK);
            PAPER
        } else {
            c.fill_rect(x, y, kw, 1, SECONDARY);
            c.fill_rect(x, y + kh - 1, kw, 1, SECONDARY);
            c.fill_rect(x, y, 1, kh, SECONDARY);
            c.fill_rect(x + kw - 1, y, 1, kh, SECONDARY);
            INK
        };
        draw_text_in(c, font, x + CAP_X, y + CAP_Y, key, word_ink);
        x = draw_text_in(c, font, x + kw + space, y + CAP_Y, does, INK) + gap;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::{Gray320x240, Inset, PAPER};
    use crate::font::{peep7x14, peep10x20};

    const FONTS: Fonts<'static> = Fonts {
        title: &peep10x20::FONT,
        body: &peep10x20::FONT,
        small: &peep7x14::FONT,
        gap: 2,
        margin: 6,
        colour: true,
        scrollbar: 6,
        title_rule: false,
    };

    fn rows_inked<C: Canvas + ?Sized>(c: &C) -> heapless::Vec<usize, 240> {
        let mut v = heapless::Vec::new();
        for y in 0..c.height() {
            if (0..c.width()).any(|x| c.get(x, y) != PAPER) {
                let _ = v.push(y);
            }
        }
        v
    }

    /// Every part lands on the page, top to bottom, and the keys are the last thing drawn
    /// -- nothing overlaps them, however much small print there is.
    #[test]
    fn a_page_with_everything_keeps_the_keys_on_screen() {
        let mut fb = Gray320x240::new();
        let mut c = Inset::new(&mut fb, 16);
        let small = [
            "as someone.with.a.rather.long.name@example.com",
            "Wallet: root 1A2B3C4D",
            "This wallet already has a login here.",
            "and one more line that should be dropped rather than cover the keys",
        ];
        let a = Approval {
            head: "Register",
            art: Some(&crate::art::menuicons::SECURE_KEY),
            main: "accounts.some-very-long-domain.example.com",
            small: &small,
            yes: (KeyMark::Word("ENTER"), "allow"),
            no: (KeyMark::Word("CANCEL"), "refuse"),
        };
        draw(&mut c, &FONTS, &a);
        let rows = rows_inked(&c);
        let last = *rows.last().unwrap();
        let space = FONTS.margin + 2;
        let hint_top = c.height() - keys_height(&FONTS, &a);
        assert!(last >= hint_top, "the keys are not along the bottom");
        assert!(last < c.height());
        // A band of paper just above the keys: the small print stopped short of them.
        assert!(
            (hint_top.saturating_sub(space / 2)..hint_top).all(|y| !rows.contains(&y)),
            "something runs into the keys"
        );
        // The heading is at the top.
        assert!(rows.first().copied().unwrap() < space + peep10x20::FONT.line_height());
    }

    /// Where the picture would push small print off the page, the picture goes and every
    /// line stays; where there is room for both, both are drawn.
    #[test]
    fn the_words_win_over_the_picture() {
        let art_pixels = |small: &[&str]| {
            let mut fb = Gray320x240::new();
            let mut c = Inset::new(&mut fb, 16);
            let a = Approval {
                head: "Share RNG samples?",
                art: Some(&crate::art::menuicons::ANALYZE_RNG),
                main: "Raw bytes from this device's random generators, for the paired computer.",
                small,
                yes: (KeyMark::Word("ENTER"), "share"),
                no: (KeyMark::Word("CANCEL"), "refuse"),
            };
            draw(&mut c, &FONTS, &a);
            // The art draws in the palette's levels 1 to 9; text never does.
            (0..c.height())
                .flat_map(|y| (0..c.width()).map(move |x| (x, y)))
                .filter(|&(x, y)| (1..=9).contains(&c.get(x, y)))
                .count()
        };
        assert!(art_pixels(&["Asked once."]) > 0, "room for both");
        let long = [
            "Never your seed or keys; these bytes are not used for any wallet.",
            "Asked once while this computer stays paired.",
        ];
        assert_eq!(art_pixels(&long), 0, "the picture stayed and a line went");
    }

    /// A short page draws what it has and leaves the rest of the canvas blank.
    #[test]
    fn a_short_page_is_short() {
        let mut fb = Gray320x240::new();
        let mut c = Inset::new(&mut fb, 16);
        let a = Approval {
            head: "Sign in",
            art: Some(&crate::art::menuicons::SECURE_KEY),
            main: "example.com",
            small: &[],
            yes: (KeyMark::Word("ENTER"), "allow"),
            no: (KeyMark::Word("CANCEL"), "refuse"),
        };
        draw(&mut c, &FONTS, &a);
        assert!(!rows_inked(&c).is_empty());
    }

    /// On the 128x64 panel, with no picture and the tick/cross marks, a warning whose
    /// sentence is too long for two body lines still says all of it -- in the small face
    /// -- and the keys stay along the bottom.
    #[test]
    fn a_mono_page_keeps_the_whole_warning() {
        use crate::font::misc4x6;
        use crate::framebuffer::Mono128x64;
        let mono = Fonts {
            title: &peep7x14::FONT,
            body: &peep7x14::FONT,
            small: &misc4x6::FONT,
            gap: 1,
            margin: 2,
            colour: false,
            scrollbar: 3,
            title_rule: false,
        };
        let text = "Every site this wallet registered with stops accepting it.";
        let (_, cut) = wrap::<_, 3>(&misc4x6::FONT, text, 128 - 4);
        assert!(!cut, "the small face should fit it in three lines");
        let mut fb = Mono128x64::new();
        let a = Approval {
            head: "Reset security key?",
            art: None,
            main: text,
            small: &["This cannot be undone."],
            yes: (KeyMark::Icon(&crate::icons::CHECK), "reset"),
            no: (KeyMark::Icon(&crate::icons::CROSS), "keep it"),
        };
        draw(&mut fb, &mono, &a);
        let rows = rows_inked(&fb);
        assert!(
            *rows.last().unwrap() >= 64 - 7,
            "the keys are not along the bottom"
        );
    }
}
