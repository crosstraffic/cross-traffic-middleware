# Changelog

## 0.3.2 — 2026-08

### Added

- **`WasmAlternativeIntersection`**: the HCM Ch.23 Part C RCUT/MUT operational analysis (Exhibit 23-47 Steps 6-10). The constructor takes a configuration object matching the library's `AlternativeIntersection` serde schema (form, movements with ordered junction journeys); results come back as per-movement Equation 23-60 ETT/LOS rows plus the Equation 23-61/23-62 approach and intersection aggregations, with intersection LOS from Exhibit 23-13.
- **Part C helper functions**, exported as plain functions: `edtt_merge` (Equation 23-58) and `edtt_stop_or_signal` (Equation 23-59) so the UI can compute extra distance travel time from crossover geometry, `uturn_saturation_adjustment` (Exhibit 23-52), `stop_junction_delay` (the Chapter 20 gap-acceptance capacity/delay/queue bundle used at STOP-controlled crossovers), and `dlt_offset` (the DLT supplemental-intersection offset, Equations 23-63 through 23-68). Together with the existing `WasmDisplacedLeftTurn` (Equation 23-69) this completes the wasm surface for the Part C forms; the library implementations are validated against Chapter 34 Example Problems 12-16.

HCM-coverage releases stay within the 0.3.x family so the CrossTraffic crates carry a matched version line.

### Added

- **Selectable HCM edition on `WasmRampSegment`**, completing what 0.3.0 did for weaving. A `version` property ("7", default, or "7.1") and a trailing constructor argument. `run_analysis()` dispatches on the edition; `analysis_v7_1()` returns the full Edition 7.1 result as a JS object (null until an analysis has run), with `speed_avg` null when demand sits far past capacity and the speed equation loses physical meaning.

### Changed

- **The `WasmRampSegment` stepwise methods throw on a "7.1" segment** instead of silently returning 7th Edition numbers — Edition 7.1 has no lane-distribution model, so v_12 and the Exhibit 14-13 speeds do not exist there. The Rust signatures gain `Result` (breaking for direct Rust callers of the wasm wrapper, of which the only known one is the web calculator's re-export crate); JS behavior is unchanged for 7th Edition segments.

## 0.3.0 — 2026-08

### Added

- **Selectable HCM edition on `WasmWeavingSegment`.** A `version` property ("7", default, or "7.1"), a trailing constructor argument, and the `nw_rf`/`nw_fr`/`nw_rr` weaving-lane counts the Edition 7.1 configuration weighting reads. `run_analysis()` dispatches on the edition; `analysis_v7_1()` returns the full Edition 7.1 result as a JS object (null until an analysis has run).

### Breaking

- **The `WasmWeavingSegment` stepwise methods throw on a "7.1" segment** instead of silently returning 7th Edition numbers — the editions disagree on speeds, capacities, and LOS bands. Rust signatures gain `Result`, hence the version bump; JS behavior is unchanged for 7th Edition segments.

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
