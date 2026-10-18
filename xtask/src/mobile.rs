//! `cargo xtask mobile [--write|--check]`: progress of the mobile port from the checklist in
//! `docs/mobile-roadmap.md`.
//!
//! Each phase is a `## M<n> — name · weight <w>` heading followed by `- [x]` / `- [ ]` items. A
//! phase is done/total items; the overall figure weights the phases by `<w>`. `--write` rewrites
//! the table between the `progress:start` / `progress:end` markers, `--check` fails when that table
//! is stale (so the doc cannot drift from its own checklist).

use std::path::Path;

const DOC: &str = "docs/mobile-roadmap.md";
const START: &str = "<!-- progress:start -->";
const END: &str = "<!-- progress:end -->";

#[derive(Debug, PartialEq)]
pub struct Phase {
    pub id: String,
    pub name: String,
    pub weight: u32,
    pub done: u32,
    pub total: u32,
}

impl Phase {
    pub fn percent(&self) -> u32 {
        if self.total == 0 { 0 } else { (self.done as f64 * 100.0 / self.total as f64).round() as u32 }
    }
}

/// `## M3 — Media and storage · weight 15` → (`M3`, `Media and storage`, 15).
fn heading(line: &str) -> Option<(String, String, u32)> {
    let rest = line.strip_prefix("## ")?;
    let (title, weight) = rest.rsplit_once("· weight ")?;
    let weight = weight.trim().parse().ok()?;
    let (id, name) = title.split_once('—')?;
    let id = id.trim();
    if !id.starts_with('M') || id.len() < 2 || !id[1..].chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some((id.to_string(), name.trim().to_string(), weight))
}

pub fn parse(doc: &str) -> Vec<Phase> {
    let mut phases: Vec<Phase> = Vec::new();
    for line in doc.lines() {
        if let Some((id, name, weight)) = heading(line) {
            phases.push(Phase { id, name, weight, done: 0, total: 0 });
        } else if line.starts_with("## ") {
            // any other section ends the phase
            phases.push(Phase { id: String::new(), name: String::new(), weight: 0, done: 0, total: 0 });
        } else if let Some(p) = phases.last_mut() {
            let t = line.trim_start();
            if t.starts_with("- [x] ") || t.starts_with("- [X] ") {
                p.done += 1;
                p.total += 1;
            } else if t.starts_with("- [ ] ") {
                p.total += 1;
            }
        }
    }
    phases.retain(|p| !p.id.is_empty());
    phases
}

/// Weighted overall percent (0 when no phase has a weight).
pub fn overall(phases: &[Phase]) -> u32 {
    let weight: u32 = phases.iter().map(|p| p.weight).sum();
    if weight == 0 {
        return 0;
    }
    let sum: f64 = phases.iter().map(|p| p.weight as f64 * if p.total == 0 { 0.0 } else { p.done as f64 / p.total as f64 }).sum();
    (sum * 100.0 / weight as f64).round() as u32
}

fn bar(percent: u32) -> String {
    let filled = (percent as usize).min(100) / 10;
    format!("{}{}", "█".repeat(filled), "░".repeat(10 - filled))
}

pub fn table(phases: &[Phase]) -> String {
    let mut out = String::from("| Phase | Weight | Done | Progress |\n|---|---|---|---|\n");
    for p in phases {
        out.push_str(&format!("| {} {} | {} | {}/{} | {} {}% |\n", p.id, p.name, p.weight, p.done, p.total, bar(p.percent()), p.percent()));
    }
    let done: u32 = phases.iter().map(|p| p.done).sum();
    let total: u32 = phases.iter().map(|p| p.total).sum();
    let o = overall(phases);
    out.push_str(&format!("| **Overall (weighted)** | {} | {done}/{total} | **{} {o}%** |\n", phases.iter().map(|p| p.weight).sum::<u32>(), bar(o)));
    out
}

/// `doc` with the block between the markers replaced by `table`.
pub fn replace_table(doc: &str, table: &str) -> Result<String, String> {
    let start = doc.find(START).ok_or_else(|| format!("{DOC}: missing {START}"))?;
    let end = doc.find(END).ok_or_else(|| format!("{DOC}: missing {END}"))?;
    if end < start {
        return Err(format!("{DOC}: {END} comes before {START}"));
    }
    Ok(format!("{}{START}\n{table}{}", &doc[..start], &doc[end..]))
}

pub fn run(root: &Path, args: &[&str]) -> Result<(), String> {
    let path = root.join(DOC);
    let doc = std::fs::read_to_string(&path).map_err(|e| format!("{DOC}: {e}"))?;
    let phases = parse(&doc);
    if phases.is_empty() {
        return Err(format!("{DOC}: no `## M<n> — name · weight <w>` phases found"));
    }
    let table = table(&phases);
    let updated = replace_table(&doc, &table)?;
    if args.contains(&"--write") {
        if updated != doc {
            std::fs::write(&path, updated).map_err(|e| format!("{DOC}: {e}"))?;
        }
    } else if args.contains(&"--check") && updated != doc {
        return Err(format!("{DOC}: the progress table is stale; run `cargo xtask mobile --write`"));
    }
    print!("{table}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOC: &str = "# T\n\n## M0 — Base · weight 10\n- [x] a\n- [x] b\n- [ ] c\n- [ ] d\n\n## M1 — Shell · weight 30\n  - [x] nested\n- [ ] e\n\n## Notes\n- [x] not a phase item\n\n<!-- progress:start -->\nold\n<!-- progress:end -->\n";

    #[test]
    fn counts_items_per_phase_and_ignores_other_sections() {
        let p = parse(DOC);
        assert_eq!(p.len(), 2);
        assert_eq!((p[0].id.as_str(), p[0].done, p[0].total, p[0].percent()), ("M0", 2, 4, 50));
        assert_eq!((p[1].done, p[1].total, p[1].percent()), (1, 2, 50));
    }

    #[test]
    fn overall_is_weighted() {
        let mut p = parse(DOC);
        assert_eq!(overall(&p), 50);
        p[0].done = 4; // 100 % at weight 10, 50 % at weight 30 → (10 + 15) / 40
        assert_eq!(overall(&p), 63);
    }

    #[test]
    fn empty_phases_and_zero_weights_do_not_divide_by_zero() {
        assert_eq!(overall(&[]), 0);
        let p = vec![Phase { id: "M0".into(), name: "x".into(), weight: 0, done: 0, total: 0 }];
        assert_eq!(overall(&p), 0);
        assert_eq!(p[0].percent(), 0);
    }

    #[test]
    fn replaces_only_the_marked_block() {
        let out = replace_table(DOC, "TABLE\n").unwrap();
        assert!(out.contains("<!-- progress:start -->\nTABLE\n<!-- progress:end -->"));
        assert!(out.starts_with("# T"));
        assert!(replace_table("no markers", "x").is_err());
    }

    #[test]
    fn a_bad_heading_is_not_a_phase() {
        assert_eq!(heading("## M2 — Name · weight x"), None);
        assert_eq!(heading("## Mx — Name · weight 5"), None);
        assert_eq!(heading("## M2 — Name · weight 5"), Some(("M2".into(), "Name".into(), 5)));
    }
}
