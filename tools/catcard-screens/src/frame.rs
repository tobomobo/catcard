//! One Q1 frame, put together the way the firmware's display layer puts it together.
//!
//! The firmware draws every screen into a 4-bit [`Gray320x240`] and turns levels into
//! colours only as it flushes, through two palettes: the status bar's rows through
//! [`GREYS`], the rest through whatever the screen asked for. Full-colour marks (a chain's
//! logo in a list row) never touch the canvas; they are composited into their rows as
//! the rows go out. This module does the same three things and stops where the SPI
//! would start, handing back RGB888 instead.
//!
//! **This mirrors `crates/catcard-fw/src/display.rs`, it does not share it.** That file
//! only builds for the device. The constants below are copies of the Q1 values there,
//! each naming what it copies; when one changes there, change it here.

use catcard_ui::art::rgba;
use catcard_ui::canvas::{Canvas, Gray320x240, Inset, Level};
use catcard_ui::font::{peep7x14, peep10x20};
use catcard_ui::scroll::{Fonts, ScrollView};
use catcard_ui::st7789::{GREYS, HEIGHT, WIDTH};
use catcard_ui::statusbar::{self, Status};
use catcard_ui::theme::{LIST, SECONDARY};
use catcard_ui::widgets::Layout;

/// `display::BAR_H` on the Q1: the status bar's rows.
pub const BAR_H: usize = 16;
/// `display::SCREEN_W` on the Q1.
pub const SCREEN_W: usize = WIDTH;
/// `display::SCREEN_H` on the Q1: the panel less the bar.
pub const SCREEN_H: usize = HEIGHT - BAR_H;
/// `display::LAYOUT` on the Q1.
pub const LAYOUT: Layout<'static> = Layout::roomy();
/// `display::FONTS` on the Q1.
pub const FONTS: Fonts<'static> = Fonts {
    title: &peep10x20::FONT,
    body: &peep10x20::FONT,
    small: &peep7x14::FONT,
    gap: 2,
    margin: 6,
    colour: true,
    scrollbar: 6,
    title_rule: true,
};
/// `display::STRIP_INK` on the Q1: the help strip's label on an ordinary frame.
pub const STRIP_INK: Level = SECONDARY;
/// `menu::GRID_STRIP_INK`: the help strip's label under the icon grid.
pub const GRID_STRIP_INK: Level = SECONDARY;
/// `display::MARK_SIDE`: the largest colour mark the firmware composites.
const MARK_SIDE: usize = 24;

/// `display::Strip`: whether a frame has the help strip along its foot.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Strip {
    Off,
    Shown,
    Focused,
}

impl Strip {
    /// `display::Strip::body_h`: what is left above the strip.
    pub fn body_h(self) -> usize {
        SCREEN_H - self.rows()
    }

    fn rows(self) -> usize {
        match self {
            Strip::Off => 0,
            _ => catcard_ui::helpstrip::height(FONTS.small),
        }
    }
}

/// `display::strip_frame`: the content in the band above the strip, then the strip.
pub fn strip_frame<C: Canvas + ?Sized>(
    c: &mut C,
    strip: Strip,
    ink: Level,
    f: impl FnOnce(&mut Inset<'_, C>),
) {
    if strip == Strip::Off {
        f(&mut Inset::band(c, 0, 0));
        return;
    }
    let look = catcard_ui::helpstrip::Strip {
        focused: strip == Strip::Focused,
        ink,
    };
    catcard_ui::helpstrip::frame(c, FONTS.small, look, f);
}

/// The surface a screen draws on: the panel below the bar.
pub type Surface<'a> = Inset<'a, Gray320x240>;

/// Full-colour pixels laid over a rectangle of the frame, RGB565.
struct Overlay {
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    pixels: Vec<u16>,
}

/// A finished frame: the canvas, the palettes it goes out through, and any marks.
pub struct Frame {
    canvas: Box<Gray320x240>,
    /// Rows above `split` go out through `top`, the rest through `body`.
    top: [u16; 16],
    body: [u16; 16],
    split: usize,
    overlays: Vec<Overlay>,
}

impl Frame {
    fn blank() -> Self {
        Self {
            canvas: Box::new(Gray320x240::new()),
            top: GREYS,
            body: LIST,
            split: 0,
            overlays: Vec::new(),
        }
    }

    /// `display::draw`: an ordinary screen, in [`LIST`], under the status bar.
    pub fn draw(status: &Status, f: impl FnOnce(&mut Surface<'_>)) -> Self {
        Self::draw_with(&LIST, status, f)
    }

    /// `display::draw_with`: as [`draw`](Self::draw), the content through `palette`.
    /// The bar keeps its own greys.
    pub fn draw_with(
        palette: &[u16; 16],
        status: &Status,
        f: impl FnOnce(&mut Surface<'_>),
    ) -> Self {
        let mut fr = Self::blank();
        {
            let mut surface = Inset::new(&mut *fr.canvas, BAR_H);
            f(&mut surface);
        }
        statusbar::render(&mut *fr.canvas, FONTS.small, status);
        fr.top = GREYS;
        fr.body = *palette;
        fr.split = BAR_H;
        fr
    }

    /// `display::draw_with_palette`: the whole panel through one palette, no bar. The
    /// splash and the artwork screens.
    pub fn draw_with_palette(palette: &[u16; 16], f: impl FnOnce(&mut Gray320x240)) -> Self {
        let mut fr = Self::blank();
        f(&mut fr.canvas);
        fr.top = *palette;
        fr.body = *palette;
        fr.split = 0;
        fr
    }

    /// `display::draw_list`: a scrolling list or document, its colour marks, and the help
    /// strip under it. `view` must have been built [`Strip::body_h`] tall.
    pub fn draw_list(status: &Status, view: &ScrollView<'_>, strip: Strip) -> Self {
        let mut fr = Self::draw(status, |c| {
            strip_frame(c, strip, STRIP_INK, |b| catcard_ui::scroll::render(b, view))
        });
        // `display::draw_with_marks`: each logo blended onto its row's own background,
        // white for the selected row and the ramp's floor otherwise.
        for shown in view.marks_shown() {
            let (w, h) = (shown.art.width as usize, shown.rows);
            if w > MARK_SIDE || h > MARK_SIDE {
                continue;
            }
            let bg = if shown.selected { LIST[15] } else { LIST[0] };
            let mut pixels = vec![bg; w * h];
            let skip = shown.skip;
            let _ = rgba::decode(shown.art, |x, y, p| {
                if let Some(row) = y.checked_sub(skip).filter(|&r| r < h) {
                    pixels[row * w + x] = rgba::over(p, bg);
                }
            });
            fr.overlays.push(Overlay {
                x: shown.x,
                y: shown.y + BAR_H,
                w,
                h,
                pixels,
            });
        }
        fr
    }

    /// The panel as RGB565, one value per pixel, row-major: what the ST7789 is sent.
    fn rgb565(&self) -> Vec<u16> {
        let mut out = Vec::with_capacity(WIDTH * HEIGHT);
        for y in 0..HEIGHT {
            let palette = if y < self.split {
                &self.top
            } else {
                &self.body
            };
            for x in 0..WIDTH {
                out.push(palette[self.canvas.get(x, y) as usize]);
            }
        }
        for o in &self.overlays {
            for dy in 0..o.h {
                for dx in 0..o.w {
                    let (x, y) = (o.x + dx, o.y + dy);
                    if x < WIDTH && y < HEIGHT {
                        out[y * WIDTH + x] = o.pixels[dy * o.w + dx];
                    }
                }
            }
        }
        out
    }

    /// The frame as RGB888 at `scale` panel pixels per side, for a PNG.
    pub fn rgb888(&self, scale: usize) -> (usize, usize, Vec<u8>) {
        let scale = scale.max(1);
        let (w, h) = (WIDTH * scale, HEIGHT * scale);
        let src = self.rgb565();
        let mut out = Vec::with_capacity(w * h * 3);
        for y in 0..h {
            for x in 0..w {
                let p = src[(y / scale) * WIDTH + x / scale];
                // Each channel stretched to the full byte, as the panel shows it.
                let r = ((p >> 11) & 0x1F) as u32 * 255 / 31;
                let g = ((p >> 5) & 0x3F) as u32 * 255 / 63;
                let b = (p & 0x1F) as u32 * 255 / 31;
                out.extend_from_slice(&[r as u8, g as u8, b as u8]);
            }
        }
        (w, h, out)
    }
}
