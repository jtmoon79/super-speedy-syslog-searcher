---
name: sync-rust-platform-tiers
description: Update tools/cross-builds.sh TIER_TARGETS from Rust 1.88.0 platform support tiers
agent: agent
argument-hint: Optional: rustc doc version URL or notes
---
Update the TIER_TARGETS array in [tools/cross-builds.sh](../../tools/cross-builds.sh) from the Rust platform support source.

Requirements:
- Read tiers and targets from https://doc.rust-lang.org/1.88.0/rustc/platform-support.html or its canonical markdown source.
- Include every listed target triple from Tier 1, Tier 2 with host tools, Tier 2 without host tools, and Tier 3.
- Emit each entry as "<tier>${SEP}<target-triple>" in TIER_TARGETS.
- Keep the rest of the script unchanged.
- Validate with a shell syntax check on tools/cross-builds.sh.
- Report entry count and a short diff summary.

If extra user arguments are provided, apply them as additional constraints.
