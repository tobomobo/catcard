//! The screens, each built the way the firmware builds it, from sample data.
//!
//! A scene names the firmware function it mirrors. The firmware composes its screens in
//! `catcard-fw`, which only builds for the device, so each one is re-assembled here from
//! the same `catcard-ui` calls. The widgets are the real ones; the composition is a
//! copy, and drifts if the firmware's does.
//!
//! Add a scene by adding a function and a row to [`ALL`].

use catcard_ui::art::{chainicons, menuicons, tibane};
use catcard_ui::field::{self, Field, Skin};
use catcard_ui::grid::{self, CELLS, Cell, ROWS};
use catcard_ui::icons::KeyMark;
use catcard_ui::pinpage::{self, PinPage};
use catcard_ui::scroll::{Line, ScrollView};
use catcard_ui::statusbar::{Charge, Power, Status};
use catcard_ui::theme::PAGE;

use crate::frame::{FONTS, Frame, GRID_STRIP_INK, LAYOUT, SCREEN_H, SCREEN_W, Strip, strip_frame};

pub struct Scene {
    pub name: &'static str,
    pub about: &'static str,
    pub render: fn() -> Frame,
}

pub const ALL: &[Scene] = &[
    Scene {
        name: "splash",
        about: "Power-on splash, artwork palette, no bar (splash.rs)",
        render: splash,
    },
    Scene {
        name: "home",
        about: "Main menu as an icon grid, seed loaded (menu.rs draw_grid)",
        render: home,
    },
    Scene {
        name: "home-blank",
        about: "Main menu on a blank device (MAIN_ITEMS_BLANK)",
        render: home_blank,
    },
    Scene {
        name: "home-help-focus",
        about: "Main menu with the help strip focused",
        render: home_help_focus,
    },
    Scene {
        name: "home-passphrase",
        about: "Main menu with a passphrase in force and SHIFT held",
        render: home_passphrase,
    },
    Scene {
        name: "utils-grid",
        about: "A grid longer than a page: edge hint and page dots",
        render: utils_grid,
    },
    Scene {
        name: "settings",
        about: "A list menu with the help strip (menu.rs build_menu_view)",
        render: settings,
    },
    Scene {
        name: "chains",
        about: "Toggle list with full-colour chain marks (menu.rs, Chains)",
        render: chains,
    },
    Scene {
        name: "pin-prefix",
        about: "PIN entry, first half (pinentry.rs screen_pin)",
        render: pin_prefix,
    },
    Scene {
        name: "pin-words",
        about: "PIN entry, anti-phishing words shown (pinentry.rs screen_pin)",
        render: pin_words,
    },
    Scene {
        name: "pin-retry",
        about: "PIN entry after a wrong PIN: tries left",
        render: pin_retry,
    },
    Scene {
        name: "pin-scrambled",
        about: "PIN second half with the scrambled number row",
        render: pin_scrambled,
    },
    Scene {
        name: "pin-checking",
        about: "PIN first half accepted, words on their way",
        render: pin_checking,
    },
    Scene {
        name: "approval",
        about: "Yes/no approval page (rngshare.rs)",
        render: approval,
    },
    Scene {
        name: "working",
        about: "Busy screen with its bar (widgets::working)",
        render: working,
    },
    Scene {
        name: "message",
        about: "Two-line message (widgets::message)",
        render: message,
    },
    Scene {
        name: "info",
        about: "Titled info lines (menu.rs info, the Boot screen)",
        render: info,
    },
    Scene {
        name: "seed-words",
        about: "Writing down 12 words, two columns (menu.rs show_words)",
        render: seed_words,
    },
    Scene {
        name: "seed-words-24",
        about: "A 24-word backup: two blocks of twelve, scrolled",
        render: seed_words_24,
    },
    Scene {
        name: "tx-review",
        about: "Reviewing a transaction before signing (signtx.rs review_page)",
        render: tx_review,
    },
    Scene {
        name: "tx-review-end",
        about: "The same review scrolled to its end: change, and ENTER now signs",
        render: tx_review_end,
    },
    Scene {
        name: "tx-big",
        about: "A 20-output batch, page 1 of 3: the totals, then outputs 1-8",
        render: tx_big_top,
    },
    Scene {
        name: "tx-big-mid",
        about: "The batch's first page, half way down",
        render: tx_big_mid,
    },
    Scene {
        name: "tx-big-page-end",
        about: "The end of the batch's first page: ENTER turns to the next",
        render: tx_big_page_end,
    },
    Scene {
        name: "tx-big-last",
        about: "The end of the batch's last page: the change, and ENTER signs",
        render: tx_big_last,
    },
    Scene {
        name: "tx-big-change",
        about: "The batch's second page: change mid-batch, with an unusual-path warning",
        render: tx_big_change,
    },
    Scene {
        name: "tx-consolidate",
        about: "A consolidation: every output is change, nothing sent",
        render: tx_consolidate,
    },
    Scene {
        name: "address-qr",
        about: "An address as a QR beside its text (menu.rs qr_screen_bytes)",
        render: address_qr,
    },
    Scene {
        name: "note-title",
        about: "Typing a new note's title (notes.rs read_into)",
        render: note_title,
    },
    Scene {
        name: "note-body",
        about: "Typing a new note's body, five lines (notes.rs read_into)",
        render: note_body,
    },
    Scene {
        name: "note-password",
        about: "A saved password's detail page (notes.rs open)",
        render: note_password,
    },
];

/// A device with its seed loaded and nothing unusual about it.
const STATUS: Status = Status {
    shift: false,
    symbol: false,
    caps: false,
    key: "MASTER",
    key_set: false,
    fingerprint: Some([0x0f, 0x05, 0x6e, 0x8c]),
    power: Some(Power::Battery(Some(Charge::ThreeQuarters))),
};

/// A blank device: no seed, so no key word and no fingerprint.
const STATUS_BLANK: Status = Status {
    key: "",
    fingerprint: None,
    ..STATUS
};

/// The Q1's main menu with a seed (`menu::MAIN_ITEMS`, Q1 rows).
const MAIN: &[&str] = &["Sign", "Addresses", "Notes", "Utils", "Derive", "Settings"];

/// `menu::grid_icon`, for the labels the scenes use.
fn grid_icon(label: &str) -> Option<&'static catcard_ui::art::indexed::Indexed> {
    use menuicons as art;
    Some(match label {
        "Sign" => &art::SIGN,
        "New" => &art::NEW_PASSPHRASE,
        "Import" => &art::IMPORT_PASSPHRASE,
        "Addresses" => &art::ADDRESS_LIST,
        "Notes" => &art::NOTES,
        "Utils" => &art::UTILS,
        "Settings" => &art::SETTINGS,
        "Scan QR" => &art::SCAN_QR_CODE,
        "Derive" => &art::DERIVE_KEY,
        "Analyze RNG" => &art::ANALYZE_RNG,
        "USB Drive" => &art::USB_DRIVE,
        "Export wallet" => &art::EXPORT_WALLET,
        "Browse files" => &art::MICROSD_BROWSE,
        "Format" => &art::MICROSD_FORMAT,
        "SD card" => &art::CARD_DETAILS,
        "Delete PSBTs" => &art::DELETE_PSBTS,
        "Paper wallet" => &art::PAPER_WALLET,
        "Backup" => &art::BACKUP,
        "Upgrade Firmware" => &art::FIRMWARE_UPGRADE,
        _ => return None,
    })
}

/// `menu::draw_grid`, for a frame that has not slid: the cursor's page of icons through
/// the art's palette, with the help strip under it.
fn grid(status: &Status, items: &[&str], cursor: usize, strip: Strip) -> Frame {
    let off = grid::window(cursor, items.len(), 0);
    Frame::draw_with(&PAGE, status, |c| {
        strip_frame(c, strip, GRID_STRIP_INK, |band| {
            let (col, row) = grid::place(cursor);
            let cells: Vec<Cell<'_>> = items
                .iter()
                .skip(off * ROWS)
                .take(CELLS)
                .map(|label| Cell {
                    label,
                    icon: grid_icon(label),
                })
                .collect();
            let within = col.saturating_sub(off) * ROWS + row;
            grid::render_page(
                band,
                LAYOUT.title,
                LAYOUT.body,
                &cells,
                within,
                off,
                items.len(),
            );
        })
    })
}

fn splash() -> Frame {
    Frame::draw_with_palette(&tibane::LOGO.palette, |c| {
        catcard_ui::splash::draw_colour(c, &tibane::LOGO, "7.0.0", 40, &LAYOUT)
    })
}

fn home() -> Frame {
    grid(&STATUS, MAIN, 0, Strip::Shown)
}

fn home_blank() -> Frame {
    grid(
        &STATUS_BLANK,
        &["New", "Import", "Scan QR", "Utils", "Settings"],
        1,
        Strip::Shown,
    )
}

fn home_help_focus() -> Frame {
    grid(&STATUS, MAIN, 0, Strip::Focused)
}

fn home_passphrase() -> Frame {
    let status = Status {
        key: "PASSPHRASE",
        key_set: true,
        shift: true,
        fingerprint: Some([0xa1, 0x7c, 0x33, 0x02]),
        ..STATUS
    };
    grid(&status, MAIN, 3, Strip::Shown)
}

fn utils_grid() -> Frame {
    grid(
        &STATUS,
        &[
            "Analyze RNG",
            "USB Drive",
            "Export wallet",
            "Browse files",
            "Format",
            "SD card",
            "Delete PSBTs",
            "Paper wallet",
            "Backup",
            "Upgrade Firmware",
        ],
        2,
        Strip::Shown,
    )
}

/// `menu::build_menu_view` and `display::draw_list`.
/// `menu::build_menu_view`, rows with an optional value at their right.
fn list(title: &str, note: &str, items: &[(&str, Option<&str>)], cursor: usize) -> Frame {
    let mut lines = vec![Line::title(title)];
    if !note.is_empty() {
        lines.push(Line::body(note).small().centered().muted());
    }
    for (i, (item, now)) in items.iter().enumerate() {
        let mut line = Line::item(item, i as u32).wrapped();
        if let Some(now) = now {
            line = line.aside(now);
        }
        lines.push(line);
    }
    let strip = Strip::Shown;
    let mut view = ScrollView::build(&lines, SCREEN_W, strip.body_h(), FONTS);
    view.select(cursor as u32);
    Frame::draw_list(&STATUS, &view, strip)
}

/// `menu::settings_items` with `menu::setting_now`: each value-holding row says what it
/// is set to at its right. Sample values, the defaults.
fn settings() -> Frame {
    list(
        "Settings",
        "",
        &[
            ("Login", None),
            ("Passphrase", None),
            ("Multisig", None),
            ("Spending Policy", None),
            ("Idle timeout", Some("Off")),
            ("Display units", Some("BTC")),
            ("Max network fee", Some("10%")),
            ("SLIP-132 export", Some("Off")),
            ("Hardware On/Off", None),
            ("NFC Push Tx", Some("Off")),
            ("Menu wrapping", Some("On")),
            ("LCD brightness", Some("100%")),
        ],
        4,
    )
}

/// The chain toggles in `menu.rs`: large rows, each with its logo and a switch.
fn chains() -> Frame {
    let rows = [
        ("Bitcoin", "BTC", true),
        ("Ethereum", "ETH", true),
        ("Solana", "SOL", false),
        ("Litecoin", "LTC", false),
        ("Bitcoin Cash", "BCH", false),
    ];
    let mut lines = vec![Line::title("Chains")];
    for (i, (name, ticker, on)) in rows.iter().enumerate() {
        let mut line = Line::item(name, i as u32).large().toggled(*on);
        if let Some(mark) = chainicons::mark(ticker) {
            line = line.with_mark(mark);
        }
        lines.push(line);
    }
    let mut view = ScrollView::build(&lines, SCREEN_W, SCREEN_H, FONTS);
    view.select(1);
    Frame::draw_list(&STATUS, &view, Strip::Off)
}

/// `pinentry::screen_pin`: the page is `catcard_ui::pinpage`, drawn through the typing
/// palette.
fn pin_page(p: PinPage<'_>) -> Frame {
    Frame::draw_with(&PAGE, &STATUS_BLANK, |c| {
        pinpage::draw(c, &LAYOUT, &p);
    })
}

/// The first half, three digits in.
const PIN: PinPage<'static> = PinPage {
    typed: 3,
    first: 4,
    max: 6,
    min: 2,
    words: None,
    busy: false,
    caret: true,
    tries: None,
    legend: None,
};

fn pin_prefix() -> Frame {
    pin_page(PIN)
}

fn pin_words() -> Frame {
    pin_page(PinPage {
        typed: 0,
        words: Some(["orbit", "lantern"]),
        ..PIN
    })
}

fn pin_retry() -> Frame {
    pin_page(PinPage {
        typed: 0,
        tries: Some((11, 13)),
        ..PIN
    })
}

fn pin_checking() -> Frame {
    pin_page(PinPage {
        busy: true,
        caret: false,
        first: PIN.typed,
        ..PIN
    })
}

fn pin_scrambled() -> Frame {
    pin_page(PinPage {
        typed: 1,
        words: Some(["orbit", "lantern"]),
        legend: Some(("1 2 3 4 5 6 7 8 9 0", "7 3 0 1 9 4 2 8 6 5")),
        ..PIN
    })
}

fn approval() -> Frame {
    let small = [
        "Never your seed or keys; these bytes are not used for any wallet.",
        "Asked once while this computer stays paired.",
    ];
    let page = catcard_ui::approval::Approval {
        head: "Share RNG samples?",
        art: Some(&menuicons::ANALYZE_RNG),
        main: "Raw bytes from this device's random generators, for the paired computer.",
        small: &small,
        yes: (KeyMark::Word("ENTER"), "share"),
        no: (KeyMark::Word("CANCEL"), "refuse"),
    };
    Frame::draw_with(&PAGE, &STATUS, |c| {
        catcard_ui::approval::draw(c, &FONTS, &page)
    })
}

fn working() -> Frame {
    Frame::draw(&STATUS, |c| {
        catcard_ui::widgets::working(c, &LAYOUT, "Signing", "please wait", 7)
    })
}

/// `menu::message` as the export flows use it: a file's path, too long for one line,
/// and the key hint.
fn message() -> Frame {
    Frame::draw(&STATUS, |c| {
        catcard_ui::widgets::message(
            c,
            &LAYOUT,
            "Exported",
            "backups/catcard-0F056E8C-2026-11-23.7z",
            "any key to go back",
        )
    })
}

/// `menu::boot_screen`: what bring-up found, as names and values.
fn info() -> Frame {
    let rows = [
        ("HAL", "ok"),
        ("DWT", "ok"),
        ("entropy", "ok, 832 bit"),
        ("board", "q1"),
    ];
    Frame::draw(&STATUS, |c| {
        catcard_ui::widgets::facts(c, &LAYOUT, "Boot", &rows)
    })
}

const WORDS: [&str; 24] = [
    "orbit", "lantern", "cactus", "velvet", "harbor", "glimpse", "tunnel", "pepper", "anchor",
    "meadow", "fossil", "ripple", "candle", "walnut", "summit", "breeze", "copper", "island",
    "marble", "thrive", "pocket", "saddle", "violin", "zebra",
];

/// `menu::word_pairs`: two numbered words a row, each block of twelve read down its left
/// column, then down its right.
fn word_pairs(words: &[&str]) -> Vec<String> {
    const ROWS: usize = 6;
    let mut out = Vec::new();
    for (b, block) in words.chunks(2 * ROWS).enumerate() {
        let rows = block.len().div_ceil(2);
        for r in 0..rows {
            let n = b * 2 * ROWS + r;
            let mut s = format!("{:2}  {:<8}", n + 1, block[r]);
            if let Some(w) = block.get(rows + r) {
                s += &format!("   {:2}  {w}", n + rows + 1);
            }
            out.push(s);
        }
    }
    out
}

/// `menu::show_words` on the Q1: the numbered words two to a row, each secret, behind
/// the ragged sensitive-line marker, a rule between blocks of twelve. Sample words, not
/// a seed.
fn words_page(words: &[&str]) -> Frame {
    let texts = word_pairs(words);
    let mut lines = vec![
        Line::title("Write these down"),
        Line::body("1: show as BitCan").small().muted(),
    ];
    for (i, t) in texts.iter().enumerate() {
        if i > 0 && i % 6 == 0 {
            lines.push(Line::divider());
        }
        lines.push(Line::body(t).secret());
    }
    let view = ScrollView::build(&lines, SCREEN_W, Strip::Off.body_h(), FONTS)
        .with_scramble(catcard_ui::pager::Scramble::new(0x5eed_1234));
    Frame::draw_list(&STATUS, &view, Strip::Off)
}

fn seed_words() -> Frame {
    words_page(&WORDS[..12])
}

fn seed_words_24() -> Frame {
    words_page(&WORDS)
}

/// `signtx::blocked`: an address in blocks of four.
fn blocked(text: &str) -> String {
    let mut out = String::new();
    for (i, ch) in text.chars().enumerate() {
        if i > 0 && i % 4 == 0 {
            out.push(' ');
        }
        out.push(ch);
    }
    out
}

/// `signtx::review_page` on the Q1, the first and only page, inside
/// `menu::scroll_choice_as`: the amount as the headline, then the figures as label and
/// value rows, then each output's row with its blocked address behind the value bar; the
/// keys pinned in the foot. Change is grey. Sample amounts in the default BTC units.
fn tx_page(at_end: bool) -> Frame {
    let to = blocked("bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq");
    let change = blocked("bc1q9h7garjt0pvkhsdwhs3njzmmvm9mvr3jqfcvmu");
    let lines = [
        Line::title("Sign transaction"),
        Line::body("Sending").small().muted(),
        Line::body("0.01500000 BTC").large(),
        Line::body("Fee").aside("0.00002100 BTC (0.14%)").small(),
        Line::body("Inputs").aside("2 of 2 ours").small(),
        Line::body("Change").aside("0.00447900 BTC").small().muted(),
        Line::divider(),
        Line::body("To #1").aside("0.01500000 BTC"),
        Line::body(&to).small().wrapped().value(),
        Line::body("Change #2").aside("0.00447900 BTC").muted(),
        Line::body(&change).small().wrapped().value().muted(),
    ];
    tx_frame(&lines, if at_end { 1.0 } else { 0.0 }, "sign")
}

/// The review in `menu::scroll_choice_as`, scrolled to `at` of the way down (0 the top, 1
/// the end), with `accept` on ENTER once the end is on screen.
fn tx_frame(lines: &[Line], at: f32, accept: &str) -> Frame {
    let (yes, no) = (KeyMark::Word("ENTER"), KeyMark::Word("CANCEL"));
    let foot = catcard_ui::approval::answers_height(&FONTS, yes, no);
    let mut view = ScrollView::build(lines, SCREEN_W, SCREEN_H, FONTS);
    view.set_height(SCREEN_H - foot);
    view.set_off((view.max_off() as f32 * at) as usize);
    Frame::draw(&STATUS, |c| {
        use catcard_ui::canvas::Canvas as _;
        {
            let mut band = catcard_ui::canvas::Inset::band(c, 0, foot);
            catcard_ui::scroll::render(&mut band, &view);
        }
        let top = c.height() - foot;
        let enter = if view.at_end() { accept } else { "more" };
        catcard_ui::approval::draw_answers(c, &FONTS, top, (yes, enter), (no, "cancel"));
    })
}

/// A payout batch: 41 inputs, 18 recipients and two change outputs, which `review` shows
/// eight outputs a page. Made-up bech32-looking addresses, so every row reads differently.
const BIG_OUTPUTS: usize = 20;

/// The batch's change outputs; the first sits mid-batch on an odd path.
const BIG_CHANGE: [usize; 2] = [11, BIG_OUTPUTS - 1];

fn big_address(i: usize) -> String {
    const SET: &[u8] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";
    let mut x = 0x9E37_79B9u32.wrapping_mul(i as u32 + 7);
    let mut out = String::from("bc1q");
    for _ in 0..38 {
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        out.push(SET[(x % 32) as usize] as char);
    }
    out
}

fn big_sats(i: usize) -> u64 {
    150_000 + (i as u64 * 7_919_731) % 9_000_000
}

fn btc(sats: u64) -> String {
    format!("{}.{:08} BTC", sats / 100_000_000, sats % 100_000_000)
}

/// `signtx::review_page` for page `page` (from 0) of the batch, at `at` of the way down:
/// the totals and the change on the first page, numbered outputs, the odd change path's
/// warning in the accent behind its output's bar, and the totals again under the last.
fn tx_big(page: usize, at: f32) -> Frame {
    const PER: usize = 8;
    let start = page * PER;
    let end = (start + PER).min(BIG_OUTPUTS);
    let last = end == BIG_OUTPUTS;
    let change_total: u64 = BIG_CHANGE.iter().map(|&i| big_sats(i)).sum();
    let sending = btc((0..BIG_OUTPUTS).map(big_sats).sum::<u64>() - change_total);
    let change_total = btc(change_total);
    let fee = "0.00031540 BTC (0.04%)";
    let mut texts: Vec<(String, String, String, bool)> = Vec::new();
    for i in start..end {
        let change = BIG_CHANGE.contains(&i);
        let head = format!("{} #{}", if change { "Change" } else { "To" }, i + 1);
        texts.push((head, btc(big_sats(i)), blocked(&big_address(i)), change));
    }
    let note = format!("outputs {}-{} of {}", start + 1, end, BIG_OUTPUTS);
    let mut lines = vec![Line::title("Sign transaction")];
    if page == 0 {
        lines.extend([
            Line::body("Sending").small().muted(),
            Line::body(&sending).large(),
            Line::body("Fee").aside(fee).small(),
            Line::body("Inputs").aside("41 of 41 ours").small(),
            Line::body("Change (2 outputs)")
                .aside(&change_total)
                .small()
                .muted(),
            Line::divider(),
        ]);
    }
    lines.push(Line::body(&note).small());
    for (i, (head, amount, address, change)) in texts.iter().enumerate() {
        let head = Line::body(head).aside(amount);
        let addr = Line::body(address).small().wrapped().value();
        if *change {
            lines.extend([head.muted(), addr.muted()]);
        } else {
            lines.extend([head, addr]);
        }
        if start + i == BIG_CHANGE[0] {
            lines.push(
                Line::body("unusual change path m/84h/0h/0h/1/4821: index far past inputs")
                    .small()
                    .wrapped()
                    .value()
                    .warn(),
            );
        }
    }
    if last && page > 0 {
        lines.extend([
            Line::divider(),
            Line::body("Sending").aside(&sending),
            Line::body("Fee").aside(fee).small(),
        ]);
    }
    tx_frame(&lines, at, if last { "sign" } else { "next page" })
}

fn tx_big_top() -> Frame {
    tx_big(0, 0.0)
}

fn tx_big_mid() -> Frame {
    tx_big(0, 0.5)
}

fn tx_big_page_end() -> Frame {
    tx_big(0, 1.0)
}

fn tx_big_change() -> Frame {
    tx_big(1, 0.45)
}

/// A consolidation: every output back to this wallet, so the headline is what comes back
/// and the fee is a share of what is moved.
fn tx_consolidate() -> Frame {
    let change = blocked("bc1q9h7garjt0pvkhsdwhs3njzmmvm9mvr3jqfcvmu");
    let lines = [
        Line::title("Sign transaction"),
        Line::body("Back to this wallet").small().muted(),
        Line::body("1.20431577 BTC").large(),
        Line::body("Fee").aside("0.00018200 BTC (0.02%)").small(),
        Line::body("Inputs").aside("23 of 23 ours").small(),
        Line::divider(),
        Line::body("Change").aside("1.20431577 BTC").muted(),
        Line::body(&change).small().wrapped().value().muted(),
    ];
    tx_frame(&lines, 0.0, "sign")
}

fn tx_big_last() -> Frame {
    tx_big(2, 1.0)
}

fn tx_review() -> Frame {
    tx_page(false)
}

fn tx_review_end() -> Frame {
    tx_page(true)
}

/// `menu::qr_screen_captioned` for a bech32 address from the explorer: the upper-cased
/// payload as a QR, the same characters beside it in blocks of four, the path across the
/// top, through the greys a camera wants.
fn address_qr() -> Frame {
    use anyd::codes::qr::{EcLevel, QrEncoder, Version};
    const MAX: Version = match Version::new(8) {
        Some(v) => v,
        None => unreachable!(),
    };
    const BUF: usize = QrEncoder::buffer_len(MAX);
    let payload = "BC1QAR0SRRR7XFKVY5L643LYDNW9RE59GTZZWF5MDQ";
    let (mut scratch, mut storage) = ([0u8; BUF], [0u8; BUF]);
    let (grid, _) = QrEncoder::new()
        .encode_text_into(payload.as_bytes(), EcLevel::M, &mut scratch, &mut storage)
        .expect("a sample address encodes");
    Frame::draw_with(&catcard_ui::st7789::GREYS, &STATUS, |c| {
        let faces = (FONTS.body, FONTS.small);
        let fits = catcard_ui::widgets::qr_with_text(
            c,
            faces,
            FONTS.gap,
            grid.width(),
            |x, y| grid.get(x, y),
            payload,
            "m/84'/0'/0'/0/0",
        );
        assert!(fits.is_some(), "the address and its QR fit the panel");
    })
}

/// `notes::read_into`: `field::page`, through the typing palette.
fn note_entry(head: &str, label: &str, text: &str, lines: usize) -> Frame {
    let fields = [Field::text("", text).lines(lines).live(true)];
    Frame::draw_with(&PAGE, &STATUS, |c| {
        field::page(
            c,
            &LAYOUT,
            head,
            label,
            &fields,
            Skin::Paper,
            true,
            "type, then ENTER",
        );
    })
}

fn note_title() -> Frame {
    note_entry("New Note", "title", "Home wifi", 1)
}

fn note_body() -> Frame {
    note_entry(
        "New Note",
        "note",
        "Router is in the hall cupboard. Guest network password is on the fridge",
        5,
    )
}

/// `notes::open` for a password: the title, small labels over each value, then actions.
fn note_password() -> Frame {
    let lines = [
        Line::title("Home wifi"),
        Line::body("user").small().muted(),
        Line::body("admin").wrapped().value(),
        Line::body("site").small().muted(),
        Line::body("192.168.1.1").wrapped().value(),
        Line::body("notes").small().muted(),
        Line::body("Router in the hall cupboard").wrapped().value(),
        Line::divider(),
        Line::item("View Password", 0).action(),
        Line::item("Send Password", 1).action(),
        Line::item("Edit Metadata", 2).action(),
    ];
    let strip = Strip::Shown;
    let mut view = ScrollView::build(&lines, SCREEN_W, strip.body_h(), FONTS);
    view.select(0);
    Frame::draw_list(&STATUS, &view, strip)
}
