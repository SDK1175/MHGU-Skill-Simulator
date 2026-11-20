# Vendored forge-rs

Copy of https://github.com/Lawrence-al/forge-rs at rev f2af0622a2a0aa291ffbd3d2f7628c0e2d96d220 (MIT, see LICENSE).
`forge-sys/imgui` (cimgui sources used only to regenerate bindings) is left out.

Local change (2026-11-19): `forge/src/allocator.rs` counts live heap bytes and the peak
(`forge::heap_stats`, `forge::heap_reset_peak`) so Skill Simulator can log its memory use.
To update forge: copy the new revision here and apply the same change again.
