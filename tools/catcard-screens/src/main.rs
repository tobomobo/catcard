//! Render CatCard's Q1 screens to PNG, on the host, with no device and no emulator.
//!
//! ```sh
//! cargo screens                    # every scene into out/screens/
//! cargo screens home pin           # only scenes whose name contains "home" or "pin"
//! cargo screens --scale 3          # bigger pictures
//! cargo screens --list             # what there is
//! ```
//!
//! Every PNG is the panel exactly as the firmware would send it -- the same widgets, the
//! same palettes, the same status bar -- scaled up by whole pixels. `index.html` beside
//! them puts every scene on one page.
//!
//! What this is for: changing how a screen looks and seeing the result in a second rather
//! than after a flash. What it is not: a test of the firmware's screen *flow*. The scenes
//! re-assemble each screen from `catcard-ui` the way `catcard-fw` does; see `scenes.rs`.

mod frame;
mod png;
mod scenes;

use std::fmt::Write as _;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use clap::Parser;

#[derive(Parser)]
#[command(about = "Render CatCard's Q1 screens to PNG")]
struct Args {
    /// Only scenes whose name contains one of these.
    filter: Vec<String>,
    /// Where the PNGs and index.html go.
    #[arg(long, default_value = "out/screens")]
    out: PathBuf,
    /// Pixels per panel pixel.
    #[arg(long, default_value_t = 2)]
    scale: usize,
    /// List the scenes and exit.
    #[arg(long)]
    list: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let chosen: Vec<&scenes::Scene> = scenes::ALL
        .iter()
        .filter(|s| args.filter.is_empty() || args.filter.iter().any(|f| s.name.contains(f)))
        .collect();
    if args.list {
        for s in scenes::ALL {
            println!("{:<18} {}", s.name, s.about);
        }
        return Ok(());
    }
    if chosen.is_empty() {
        bail!("no scene matches {:?}; --list shows them", args.filter);
    }
    if !(1..=8).contains(&args.scale) {
        bail!("--scale must be 1 to 8");
    }

    std::fs::create_dir_all(&args.out)
        .with_context(|| format!("creating {}", args.out.display()))?;
    for s in &chosen {
        let (w, h, rgb) = (s.render)().rgb888(args.scale);
        let path = args.out.join(format!("{}.png", s.name));
        std::fs::write(&path, png::encode(w, h, &rgb)?)
            .with_context(|| format!("writing {}", path.display()))?;
        println!("{}", path.display());
    }
    let index = args.out.join("index.html");
    std::fs::write(&index, contact_sheet(&chosen, args.scale))
        .with_context(|| format!("writing {}", index.display()))?;
    println!("{}", index.display());
    Ok(())
}

/// Every scene on one page, at the size it was rendered.
fn contact_sheet(chosen: &[&scenes::Scene], scale: usize) -> String {
    let (w, h) = (320 * scale, 240 * scale);
    let mut s = String::from(
        "<!doctype html><meta charset=utf-8><title>CatCard Q1 screens</title>\
         <style>body{background:#181818;color:#ddd;font:14px system-ui;margin:24px}\
         main{display:flex;flex-wrap:wrap;gap:24px}figure{margin:0}\
         img{image-rendering:pixelated;border-radius:6px;box-shadow:0 0 0 8px #000}\
         figcaption{margin-top:14px}b{color:#fff}</style><h1>CatCard Q1 screens</h1><main>",
    );
    for sc in chosen {
        let _ = write!(
            s,
            "<figure><img src=\"{n}.png\" width={w} height={h} alt=\"{n}\">\
             <figcaption><b>{n}</b> &mdash; {a}</figcaption></figure>",
            n = sc.name,
            a = sc.about,
        );
    }
    s.push_str("</main>\n");
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_scene_renders_and_names_are_unique() {
        let mut names = std::collections::HashSet::new();
        for s in scenes::ALL {
            assert!(names.insert(s.name), "{} listed twice", s.name);
            let (w, h, rgb) = (s.render)().rgb888(1);
            assert_eq!((w, h), (320, 240));
            // Something was drawn: a scene that renders one flat colour has lost its
            // content somewhere.
            assert!(
                rgb.iter().skip(3).zip(rgb.iter()).any(|(a, b)| a != b),
                "{} is blank",
                s.name
            );
        }
    }
}
