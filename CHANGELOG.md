# Changelog

## 0.2.0 — 2026-08

Tracks transportations_library 0.3.0.

### Breaking

- **`WasmRampSegment.run_analysis()` and `determine_los()` return `Option<String>`** — `undefined` in JS for a major merge operating under capacity, where the 7th Edition defines no level of service. The previous code fabricated a letter there.
- **`WasmBasicFreeways` lateral clearances (`lc_r`, `lc_l`) take `f64`, not `u32`.** The Exhibit 12-21 note interpolates for noninteger right-side clearance, and JS numbers are floats anyway; integer values read exactly the exhibit entries.

### Added

- `WasmBasicFreeways.get_ffs_adj()` and `get_breakpoint()`, so a caller can plot the Equation 12-1 speed curve without restating Exhibit 12-6. The doc comments carry the December 2022 corrections asymmetry (breakpoint from FFS_adj, base capacity from unadjusted FFS).
- `WasmBasicFreeways` exposes `sut_percentage` and E_T for the Chapter 12 specific-upgrade PCE tables.

### Notes

- Built against transportations_library 0.3.0, which applies the December 2022 corrections (Equations 12-6/12-7 capacity from unadjusted FFS, the Chapter 20 Stage II conflicting movements, and the Exhibit 20-14 swap) and adds selectable HCM Edition 7.1 for weaving and merge/diverge. The wasm wrapper does not yet expose the edition selector; segments analyze under the 7th Edition.

## 0.1.7 and earlier

Not tracked here; see the git history.
