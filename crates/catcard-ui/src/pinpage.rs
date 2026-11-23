//! The PIN prompt as a page, for a panel with room: the Q1.
//!
//! The firmware decides what the page says -- how many digits are in, whether the words
//! are back, how many tries are left -- and this draws it. Here rather than in the
//! firmware so the screen renderer (`tools/catcard-screens`) draws the same page from the
//! same code; a copy of a layout drifts from the thing it copies.
//!
//! Like [`crate::pinentry`], it never sees a digit: it is handed a count.
//!
//! # How it reads
//!
//! A PIN here is two parts, written with a dash between them, and the page draws it that
//! way: two cards side by side, `[ first ] - [ second ]`, each under its name, the whole
//! time. Only one is live -- white, with the caret in it -- and the other says where it
//! stands: still to come (a dim outline with its places), or done.
//!
//! Once the first part is sent its card keeps its dots, greyed, and the live card moves
//! right. Greyed and not ticked: **the first part is not checked.** Any first part gets
//! words -- a mistyped one gets other words -- and nothing says whether the PIN was right
//! until both parts have gone together. The page must not claim otherwise, so it never
//! says "checking" or shows a tick before then.
//!
//! The anti-phishing words get a section of their own under the cards, under their own
//! name, large: they are something the device shows the owner, not something the owner
//! typed, so they are kept out of both cards. (An earlier version put them in the first
//! card, where they read as the first part itself.) Words the owner does not know mean
//! a mistyped first part or a device that is not theirs; the hint says to go back and
//! type it again, which on this page is what CANCEL does once the second card is empty.
//!
//! Under that, what to do now, in grey, and the tries left in the accent once one has
//! been spent. When the number row is scrambled, a bar across the foot maps each key to
//! the digit it types, a cell over each key.

use crate::canvas::{Canvas, Level, PAPER};
use crate::icons::disc;
use crate::text::{centred, draw_text_in, width_of};
use crate::theme::{ACCENT, INK, RULE, SECONDARY, SLOT};
use crate::widgets::{CAPTION_GAP, Layout, header, hint};

/// What the page shows.
pub struct PinPage<'a> {
    /// Digits typed into the part being typed.
    pub typed: usize,
    /// Digits in the first part, once it has been sent: drawn greyed in its card while
    /// the second part is typed.
    pub first: usize,
    /// The most and fewest digits a part takes.
    pub max: usize,
    pub min: usize,
    /// The anti-phishing words, once the first part has been accepted: the page is then
    /// on its second part.
    pub words: Option<[&'a str; 2]>,
    /// The first part was just accepted and the words are being fetched.
    pub busy: bool,
    /// The caret's half of the blink.
    pub caret: bool,
    /// Tries left and tries in all, once one has been spent; `None` while none has.
    ///
    /// Hidden at full count deliberately: a permanent attempt counter reads as a threat
    /// on a device that is working normally.
    pub tries: Option<(u32, u32)>,
    /// The number row's labels and what each key types this time, when the keys are
    /// scrambled: one character per key, in the row's order. Spaces are ignored, so a
    /// spaced legend reads the same.
    pub legend: Option<(&'a str, &'a str)>,
}

/// Where a card stands.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Card {
    /// Being typed: white, a dot per digit, the places left, the next one ringed.
    Live { typed: usize, caret: bool },
    /// Not yet: an outline in the rule's ink, its places in it too.
    Later,
    /// Sent, with `n` digits: its dots, greyed, laid out as if the card held `of`. Not
    /// checked -- see the module docs.
    Sent { n: usize, of: usize },
}

/// Between the two cards: room for the dash that joins them.
const DASH_ROOM: usize = 24;
/// Under the cards, before what to do.
const AFTER_CARDS: usize = 10;

/// The two cards across a canvas `w` wide, side by side and the same size: each one's
/// left edge and width, first then second. `_second_live` is which one is being typed,
/// which once decided the sizes and no longer does -- the dots fit either way, and two
/// cards that never change shape are one less thing moving between the steps.
fn columns(l: &Layout<'_>, w: usize, _second_live: bool) -> [(usize, usize); 2] {
    let pad = l.margin;
    let card = w.saturating_sub(2 * pad + DASH_ROOM) / 2;
    [(pad, card), (pad + card + DASH_ROOM, card)]
}

/// How tall a card is: a line of the title face and a margin above and below, which
/// gives a dot room to read as a dot.
fn card_height(l: &Layout<'_>) -> usize {
    l.title.line_height() + 2 * l.margin
}

/// A digit's dot: the radius of a place in a card `w` wide holding `max`, as large as its
/// share allows up to [`DOT_R`].
fn dot_radius(l: &Layout<'_>, w: usize, max: usize) -> usize {
    let step = w.saturating_sub(l.margin) / max.max(1);
    (step.saturating_sub(4) / 2).clamp(2, DOT_R)
}

/// The largest a digit's dot gets: about the height of a capital in the title face.
const DOT_R: usize = 7;
/// An empty place's mark, much smaller than a digit's dot.
const SLOT_R: usize = 2;

/// The anti-phishing words' section: its name, and the two words on one line in the
/// title face. Returns the row under it.
fn words<C: Canvas + ?Sized>(c: &mut C, l: &Layout<'_>, y: usize, [a, b]: [&str; 2]) -> usize {
    let w = c.width();
    let name = "Anti-phishing words";
    draw_text_in(c, l.body, centred(l.body, name, w), y, name, SECONDARY);
    let y = y + l.body.line_height() + CAPTION_GAP;
    let face = l.title;
    let gap = 2 * face.advance(b' ');
    let span = width_of(face, a) + gap + width_of(face, b);
    let x = w.saturating_sub(span) / 2;
    let after = draw_text_in(c, face, x, y, a, INK);
    draw_text_in(c, face, after + gap, y, b, INK);
    y + face.line_height() + 3 * l.gap
}

/// Draw `p` over the whole canvas. Returns the row under the last thing drawn above the
/// key bar.
pub fn draw<C: Canvas + ?Sized>(c: &mut C, l: &Layout<'_>, p: &PinPage<'_>) -> usize {
    c.clear();
    let w = c.width();
    let mut y = header(c, l, "Enter PIN", "");

    let live = Card::Live {
        typed: p.typed,
        caret: p.caret,
    };
    let (first, second) = match p.words {
        // Where they were typed, so nothing moves but the ink.
        Some(_) => (
            Card::Sent {
                n: p.first,
                of: p.max,
            },
            live,
        ),
        None if p.busy => (
            Card::Sent {
                n: p.first,
                of: p.max,
            },
            Card::Later,
        ),
        None => (live, Card::Later),
    };

    // Each card's name over it: white over the live one, grey over the other.
    let [(lx, lw), (rx, rw)] = columns(l, w, p.words.is_some());
    let name_ink = |card: Card| {
        if matches!(card, Card::Live { .. }) {
            INK
        } else {
            SECONDARY
        }
    };
    draw_text_in(c, l.body, lx, y, "First part", name_ink(first));
    draw_text_in(c, l.body, rx, y, "Second part", name_ink(second));
    y += l.body.line_height() + CAPTION_GAP;

    let card_h = card_height(l);
    card(c, l, (lx, y, lw, card_h), p.max, first);
    card(c, l, (rx, y, rw, card_h), p.max, second);
    // The dash that makes them one PIN.
    let dash_w = DASH_ROOM / 2;
    let dash_x = lx + lw + (DASH_ROOM - dash_w) / 2;
    c.fill_rect(dash_x, y + card_h / 2 - 1, dash_w, 2, SECONDARY);
    y += card_h + AFTER_CARDS;

    // What to do now.
    let (mut a, mut b) = ([0u8; 10], [0u8; 10]);
    let range = [
        digits(&mut a, p.min as u32),
        " to ",
        digits(&mut b, p.max as u32),
        " digits, then ENTER",
    ];
    if let Some(pair) = p.words {
        y = words(c, l, y, pair);
        // What the words are for, said once: a device that shows the wrong ones may not be
        // the owner's, and the second part is what it would be after. No key is promised
        // here -- on this page CANCEL only backs out -- just the one thing not to do.
        y = if p.typed == 0 {
            hint(c, l, y, "Not yours? CANCEL, retype the first part")
        } else if p.typed < p.min {
            line(c, l, y, &range, SECONDARY)
        } else {
            hint(c, l, y, "ENTER to log in")
        };
    } else if p.busy {
        y = hint(c, l, y, "Getting your words...");
    } else if p.typed < p.min {
        y = line(c, l, y, &range, SECONDARY);
    } else {
        y = hint(c, l, y, "ENTER to see your words");
    }

    if let Some((left, of)) = p.tries {
        // Once a try is spent, that must not be missed. "<n> of <m> tries left", without
        // a formatter.
        let (mut a, mut b) = ([0u8; 10], [0u8; 10]);
        let parts = [
            digits(&mut a, left),
            " of ",
            digits(&mut b, of),
            " tries left",
        ];
        y = line(c, l, y, &parts, ACCENT);
    }

    if let (false, Some((keys, types))) = (p.busy, p.legend) {
        key_bar(c, l, keys, types);
    }
    y
}

/// One card in the rectangle `(x, y, w, h)`.
fn card<C: Canvas + ?Sized>(
    c: &mut C,
    l: &Layout<'_>,
    (x, y, w, h): (usize, usize, usize, usize),
    max: usize,
    card: Card,
) {
    let rect = (x, y, w, h);
    match card {
        Card::Live { typed, caret } => {
            c.fill_rect(x, y, w, h, INK);
            let look = (crate::field::MARK, SLOT);
            places(c, l, rect, max, typed, look, caret);
        }
        Card::Later => {
            outline(c, rect, RULE);
            places(c, l, rect, max, 0, (RULE, RULE), false);
        }
        Card::Sent { n, of } => {
            outline(c, rect, SECONDARY);
            // Only the dots: the places left are drawn in the page's own ground, which
            // is to say not at all, and there is no caret.
            places(c, l, rect, of.min(max), n, (SECONDARY, PAPER), false);
        }
    }
}

fn outline<C: Canvas + ?Sized>(c: &mut C, (x, y, w, h): (usize, usize, usize, usize), ink: Level) {
    c.fill_rect(x, y, w, 1, ink);
    c.fill_rect(x, y + h.saturating_sub(1), w, 1, ink);
    c.fill_rect(x, y, 1, h, ink);
    c.fill_rect(x + w.saturating_sub(1), y, 1, h, ink);
}

/// `max` places across a card, each an equal share of it, `typed` of them filled with a
/// dot in `ink`, and the rest a small mark in `slot`. With `caret`, the next place is a
/// ring in `ink`: where the next digit goes, blinking as a text cursor would.
fn places<C: Canvas + ?Sized>(
    c: &mut C,
    l: &Layout<'_>,
    (x, y, w, h): (usize, usize, usize, usize),
    max: usize,
    typed: usize,
    (ink, slot): (Level, Level),
    caret: bool,
) {
    let max = max.max(1);
    let pad = l.margin / 2;
    let inner = w.saturating_sub(2 * pad);
    let step = inner / max;
    let r = dot_radius(l, w, max);
    let x0 = x + pad + (inner - step * max) / 2 + step / 2;
    let cy = y + h / 2;
    let n = typed.min(max);
    for i in 0..max {
        let cx = x0 + i * step;
        if i < n {
            disc(c, cx, cy, r, ink);
        } else if caret && i == n {
            disc(c, cx, cy, r, ink);
            disc(c, cx, cy, r.saturating_sub(2), slot_ground(ink));
        } else {
            disc(c, cx, cy, SLOT_R.min(r), slot);
        }
    }
}

/// The ground a ring in `ink` is cut back to: the card's white under the dark ink of a
/// live card, the page under anything else.
fn slot_ground(ink: Level) -> Level {
    if ink == crate::field::MARK {
        INK
    } else {
        PAPER
    }
}

/// `parts` as one centred line in the body face. Returns the row under it.
fn line<C: Canvas + ?Sized>(
    c: &mut C,
    l: &Layout<'_>,
    y: usize,
    parts: &[&str],
    ink: Level,
) -> usize {
    let w: usize = parts.iter().map(|s| width_of(l.body, s)).sum();
    let mut x = c.width().saturating_sub(w) / 2;
    for s in parts {
        x = draw_text_in(c, l.body, x, y, s, ink);
    }
    y + l.body.line_height() + l.gap
}

/// How tall the scrambled row's bar is: a rule, the key's own label, what it types.
pub fn key_bar_height(l: &Layout<'_>) -> usize {
    1 + BAR_PAD + l.body.line_height() + l.title.line_height() + BAR_PAD
}

/// Air above and below the bar's two lines.
const BAR_PAD: usize = 3;

/// The scrambled number row as a bar across the foot of the page, one cell per key, in
/// the keys' own order -- the number row is the keyboard's top row, directly under the
/// panel, so each cell sits over the key it describes. In each cell, small and grey, the
/// label printed on the key; under it, large and white, the digit it types this time,
/// which is what the eye is looking for.
fn key_bar<C: Canvas + ?Sized>(c: &mut C, l: &Layout<'_>, keys: &str, types: &str) {
    let keys = keys.bytes().filter(|b| *b != b' ');
    let types = types.bytes().filter(|b| *b != b' ');
    let n = keys.clone().count().max(1);
    let (w, h) = (c.width(), c.height());
    let top = h.saturating_sub(key_bar_height(l));
    c.fill_rect(0, top, w, 1, RULE);
    let (cell_top, cell_h) = (top + 1, h.saturating_sub(top + 1));
    for (i, (k, t)) in keys.zip(types).enumerate() {
        let (x0, x1) = (i * w / n, (i + 1) * w / n);
        if i > 0 {
            c.fill_rect(x0, cell_top, 1, cell_h, RULE);
        }
        let mid = (x0 + x1) / 2;
        let (k, t) = ([k], [t]);
        let (k, t) = (
            core::str::from_utf8(&k).unwrap_or("?"),
            core::str::from_utf8(&t).unwrap_or("?"),
        );
        let ky = cell_top + BAR_PAD;
        let kx = mid.saturating_sub(width_of(l.body, k) / 2);
        draw_text_in(c, l.body, kx, ky, k, SECONDARY);
        let ty = ky + l.body.line_height();
        let tx = mid.saturating_sub(width_of(l.title, t) / 2);
        draw_text_in(c, l.title, tx, ty, t, INK);
    }
}

/// `v` in decimal, into `buf`.
fn digits(buf: &mut [u8; 10], mut v: u32) -> &str {
    let mut i = buf.len();
    loop {
        i -= 1;
        buf[i] = b'0' + (v % 10) as u8;
        v /= 10;
        if v == 0 {
            break;
        }
    }
    core::str::from_utf8(&buf[i..]).unwrap_or("?")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::{Gray320x240, Inset};

    fn page(typed: usize) -> PinPage<'static> {
        PinPage {
            typed,
            first: 4,
            max: 6,
            min: 2,
            words: None,
            busy: false,
            caret: true,
            tries: None,
            legend: None,
        }
    }

    fn count<C: Canvas>(c: &C, level: u8) -> usize {
        (0..c.height())
            .flat_map(|y| (0..c.width()).map(move |x| (x, y)))
            .filter(|&(x, y)| c.get(x, y) == level)
            .count()
    }

    fn drawn(p: &PinPage<'_>) -> Gray320x240 {
        let mut panel = Gray320x240::new();
        draw(&mut Inset::new(&mut panel, 16), &Layout::roomy(), p);
        panel
    }

    #[test]
    fn digits_reads_back() {
        let mut b = [0u8; 10];
        assert_eq!(digits(&mut b, 0), "0");
        assert_eq!(digits(&mut b, 13), "13");
        assert_eq!(digits(&mut b, u32::MAX), "4294967295");
    }

    /// The roles are on the page: a rule under the title, grey names and hint, and the
    /// places still to fill.
    #[test]
    fn the_page_has_its_hierarchy() {
        let c = drawn(&page(2));
        assert!(count(&c, RULE) > 200, "a rule across the page");
        assert!(count(&c, SECONDARY) > 0, "a name and a hint");
        assert!(count(&c, SLOT) > 0, "the empty places");
    }

    /// Fewer empty places as digits go in, and none once the part is full.
    #[test]
    fn slots_count_down() {
        let slots = |typed| count(&drawn(&page(typed)), SLOT);
        let (none, two, full) = (slots(0), slots(2), slots(6));
        assert!(none > two && two > 0);
        assert_eq!(full, 0);
    }

    /// Both parts are on the page the whole time, and the live one -- the white card --
    /// moves from left to right once the words are back.
    #[test]
    fn the_live_card_moves_right() {
        let white =
            |c: &Gray320x240, x: usize| (16..240).filter(|&y| c.get(x, y) == INK).count() > 30;
        let [(lx, _), (rx, rw)] = columns(&Layout::roomy(), 320, false);
        let first = drawn(&page(2));
        assert!(white(&first, lx + 2) && !white(&first, rx + rw / 2));
        let [(lx, lw), (rx, _)] = columns(&Layout::roomy(), 320, true);
        let second = drawn(&PinPage {
            words: Some(["orbit", "lantern"]),
            ..page(0)
        });
        assert!(!white(&second, lx + lw / 2) && white(&second, rx + 2));
    }

    /// Each card has room for six dots at full size.
    #[test]
    fn the_cards_hold_six_full_dots() {
        let l = Layout::roomy();
        for [(_, a), (_, b)] in [columns(&l, 320, false), columns(&l, 320, true)] {
            assert_eq!(a, b, "the two parts' cards differ");
            assert_eq!(dot_radius(&l, a, 6), DOT_R);
        }
    }

    /// The words are the device's, not the owner's typing: on the second part the first
    /// card holds only its tick, and the words are drawn under the cards, in full ink.
    #[test]
    fn the_words_are_not_in_the_first_card() {
        let l = Layout::roomy();
        let p = PinPage {
            words: Some(["orbit", "lantern"]),
            ..page(0)
        };
        let panel = drawn(&p);
        let mut scratch = Gray320x240::new();
        let top = 16 + header(&mut scratch, &l, "", "") + l.body.line_height() + CAPTION_GAP;
        let bottom = top + card_height(&l);
        let [(lx, lw), _] = columns(&l, 320, true);
        let ink_in_card = (top..bottom)
            .flat_map(|y| (lx..lx + lw).map(move |x| (x, y)))
            .any(|(x, y)| panel.get(x, y) == INK);
        assert!(!ink_in_card, "something white in the first part's card");
        let below = (bottom..240).any(|y| (0..320).any(|x| panel.get(x, y) == INK));
        assert!(below, "the words are not under the cards");
    }

    /// Nothing is checked until the whole PIN goes, so the page never draws the first
    /// part's card in full ink once it is sent -- no tick, no white -- whether the words
    /// are on their way or back.
    #[test]
    fn a_sent_first_part_is_greyed_not_ticked() {
        let l = Layout::roomy();
        let [(lx, lw), _] = columns(&l, 320, false);
        for p in [
            PinPage {
                busy: true,
                ..page(4)
            },
            PinPage {
                words: Some(["orbit", "lantern"]),
                ..page(0)
            },
        ] {
            let second = p.words.is_some();
            let [(lx, lw), _] = if second {
                columns(&l, 320, true)
            } else {
                [(lx, lw), (0, 0)]
            };
            let panel = drawn(&p);
            let mut scratch = Gray320x240::new();
            let top = 16 + header(&mut scratch, &l, "", "") + l.body.line_height() + CAPTION_GAP;
            let rows = top..top + card_height(&l);
            let grey = rows
                .clone()
                .flat_map(|y| (lx..lx + lw).map(move |x| (x, y)))
                .filter(|&(x, y)| panel.get(x, y) == SECONDARY)
                .count();
            let white = rows
                .flat_map(|y| (lx..lx + lw).map(move |x| (x, y)))
                .any(|(x, y)| panel.get(x, y) == INK);
            assert!(grey > 200, "the dots are not shown greyed: {grey}");
            assert!(!white, "the sent part is drawn as if live or checked");
            // And the dots stay in their card: the gap under the cards is empty.
            let bottom = top + card_height(&l);
            let spill =
                (bottom..bottom + AFTER_CARDS).any(|y| (0..320).any(|x| panel.get(x, y) != PAPER));
            assert!(!spill, "something ran out of the cards");
        }
    }

    /// The second part, the words, a spent try and a scrambled row all fit on the panel,
    /// the page clear of the bar at its foot.
    #[test]
    fn the_fullest_page_fits() {
        let mut panel = Gray320x240::new();
        let mut c = Inset::new(&mut panel, 16);
        let p = PinPage {
            words: Some(["abstract", "together"]),
            tries: Some((12, 13)),
            legend: Some(("1 2 3 4 5 6 7 8 9 0", "7 3 0 1 9 4 2 8 6 5")),
            ..page(3)
        };
        let l = Layout::roomy();
        let bottom = draw(&mut c, &l, &p);
        let room = c.height() - key_bar_height(&l);
        assert!(bottom <= room, "{bottom} rows above a bar at {room}");
    }

    /// The bar has a cell per key, each with the digit it types.
    #[test]
    fn the_key_bar_spans_the_page() {
        let mut panel = Gray320x240::new();
        let mut c = Inset::new(&mut panel, 16);
        let p = PinPage {
            legend: Some(("1234567890", "7301942865")),
            ..page(0)
        };
        draw(&mut c, &Layout::roomy(), &p);
        let row = c.height() - key_bar_height(&Layout::roomy());
        assert!(
            (0..c.width()).all(|x| c.get(x, row) == RULE),
            "a rule edge to edge"
        );
        // Nine dividers between ten cells.
        let below = row + 2;
        let dividers = (1..c.width()).filter(|&x| c.get(x, below) == RULE).count();
        assert_eq!(dividers, 9);
    }
}
