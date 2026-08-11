# Changelog

## 0.3.5 — 2026-08

The web calculator's boundary suite carried five chapters' worth of checks that could not run, each because a value the published example problem prints has no getter on the wrapper. This release adds those getters. Everything here reads a field the core already computes, so no result changes and nothing published earlier is removed or renamed.

### Added

- **`WasmFreewayFacility.get_volume_served()` and `volume_served_matrix()`**, the v_a matrix of Exhibits 25-48 and 25-56. Volume served equals segment demand only while the facility is undersaturated; once a queue forms the oversaturated engine meters what the segment can discharge, and it is volume served, not demand, that the period's speed and density follow. Without it the Example Problem 2 metering was invisible through the binding even though the speeds it produces were already exposed.
- **`WasmFreewayFacility.demand_based_los_matrix()`**, the lower table of Exhibit 25-59, reporting "F" where vd/c exceeded 1.0 and undefined elsewhere. The density-based `los_matrix()` can sit at D or E through a period whose demand exceeded capacity, because it grades what the segment delivered at the volume it served, so the two tables are reported side by side rather than merged.
- **`WasmPlanningFacility.get_delay_rate()` and `get_facility_queue_mi()`**, the Exhibit 25-92 section delay rates and the Exhibit 25-96 facility queue, with `total_queue_mi` added to `results_to_js_value()`. The delay rate is the undersaturated term ΔRU of Equation 25-47 alone, evaluated at the actual d/c even above 1.0, which is what the worked Example Problem 6 does rather than the ΔRU + ΔRO travel rate of Equation 25-49; the vertical queue is where the planning method puts oversaturation instead, which is why the two arrive together.
- **`WasmFreewayReliability.seed_total_vmt()` and `seed_num_periods()`**, the Equation 25-88 seed-file statistics. These are the denominator the Equation 25-77 incident frequencies are built on, so they are computed from the seed demand matrix and are readable before `run()`.
- **`WasmFreewayReliability` scenario-level accessors**: `scenario_months()`, `scenario_weekdays()`, `scenario_dafs()` (the Equation 25-72 demand adjustment factors), `scenario_incident_counts()`, `monthly_incident_frequencies()` (Equation 25-77, indexed January = 0), and `total_incidents()`, the last of which also joins `results_to_js_value()`. All five per-scenario vectors share the ordering of `scenario_probabilities()` and `scenario_tti_matrix()`, so a scenario is identified by its index across all of them. Together they make the scenario set inspectable rather than only its travel-time distribution, which is what lets a caller find the event-free seed-date scenario and check it against a plain Chapter 10 run of the same facility.
- **`WasmBasicFreeways.get_demand_volume()`**, the Equation 12-9 demand flow rate v_p in pc/h/ln, also added to `results_to_js_value()` as `demand_volume`. This is the abscissa of the Exhibit 12-6 speed-flow curve, so it is the quantity the already-exposed breakpoint and capacity are compared against, not the veh/h demand the constructor takes.
- **`WasmWeavingSegment.get_c_iwl()`, `get_capacity_weaving()`, `get_lc_w()`, and `get_lc_nw()`**, with the same four fields added to `results_to_js_value()`. `get_capacity_weaving()` returns undefined on a two-sided segment, where N_WL is zero and the Equations 13-7/13-8 criterion does not apply, so Equation 13-6 governs the capacity alone. The two lane-change components are separated from `get_lc_all()` because they respond to the configuration differently, LC_W through the short length and lane count and LC_NW through which branch the nonweaving intensity I_NW selects.
- **`WasmTwsc.add_conflicting_flow_override()` and `clear_conflicting_flow_overrides()`**, reaching the core's per-stage v_c,x override. The stage argument is "total", "stage1", or "stage2"; setting either stage refreshes the one-stage total as their sum, while setting "total" leaves the stages alone. The HCM text below Exhibits 20-8 through 20-16 allows these factors to be modified from field data. An unknown movement label or stage now throws at the binding rather than being dropped silently inside Step 3, which is how the core treats an override it cannot match. Note that the Chapter 32 Example Problem 3 fixture no longer carries any overrides, since the December 2022 corrections to Equations 20-14/20-15 and Exhibits 20-14/20-16 made the published Stage I and Stage II conflicting flows reproduce natively.

HCM-coverage releases stay within the 0.3.x family so the CrossTraffic crates carry a matched version line.

## 0.3.4 — 2026-08

A review of the 0.3.3 urban bindings found three ways a caller could get a wrong answer or no answer at all, and two documented claims that were not true of the code. Nothing published in 0.3.3 is removed; the constructor signature change below is invisible to JS callers passing valid input.

### Fixed

- **`WasmUrbanFacility.add_segment_from_config()` now counts a config carrying performance measures as a summary segment.** A config with any of `base_ffs_mph`, `travel_speed_mph`, `spatial_stop_rate_stops_mi`, `vc_ratio`, or `los` populated — the shape a serialized post-analysis fixture takes — used to slip past the `analyze()` guard that `add_segment_summary()` sets. `analyze()` would then run the Chapter 18 engine over the placeholder inputs sitting beside those measures and silently overwrite them. Such a segment now behaves like one added through `add_segment_summary()`: `analyze()` throws and `aggregate()` reports the supplied measures unchanged.
- **`WasmUrbanFacility.aggregate()` evaluates input-driven segments before aggregating**, which gives a facility mixing the two kinds of segment a working path for the first time. Previously `analyze()` refused the facility and directed the caller to `aggregate()`, which then failed with "base free-flow speed not computed" on the segments that had never been evaluated. Segments arriving with measures are left exactly as supplied. The `analyze()` error message now says that `aggregate()` handles the mixed case.
- **`WasmUrbanReliability`'s constructor rejects a `jan1_day_of_week` above 6** instead of clamping it to 6. The value anchors the calendar the reliability reporting period is built on, so clamping a mistaken 7 moved every Exhibit 17-6 day-of-week demand factor onto the wrong day with nothing to show for it. The Rust signature gains `Result<WasmUrbanReliability, JsValue>`, as the weaving and ramp stepwise methods did in 0.3.0 and 0.3.1; JS behavior is unchanged for values in 0 through 6.

### Corrections

- **The 0.3.3 note that snowfall "was hard-coded to zero, which silently removed the strongest weather events from the scenario stream" was wrong.** The library never reads `MonthlyWeather.total_snowfall_in`. The Chapter 29 weather procedure decides rain versus snow from the sampled temperature (Equations 29-3 and 29-4) and sizes the snow event from the precipitation columns through the snow-to-rain depth ratio of Step 7, so the snowfall column is climatological metadata that does not reach any computation. The `monthly_total_snowfall_in` argument stays, since removing a published argument would be breaking, and its doc now says it changes nothing. What actually brought the Chapter 29 Example Problem 4 wasm run onto the library's own numbers in 0.3.3 was `jan1_day_of_week`, `approach_lanes`, and `prop_left_turn_lanes`. Dropping the snowfall array from that run leaves the travel time indices bit-for-bit identical.
- **`num_oversaturated_scenarios()` was described as counting scenarios that "feed the residual queue forward".** The library's predicate flags a scenario when a boundary through movement runs over capacity (v/c > 1) or when it begins the period with a queue carried in (Q_b > 0), so the count already includes undersaturated periods inheriting a queue. The two conditions are not separable through the binding, which sees only the collapsed per-scenario flag. The doc comment now states the predicate.

HCM-coverage releases stay within the 0.3.x family so the CrossTraffic crates carry a matched version line.

## 0.3.3 — 2026-08

The urban streets bindings (Chapters 16, 17, and 18) could not express the inputs behind their own published example problems, so the boundary test suite documented several HCM answers as unreachable through wasm. This release closes those gaps. Every new constructor and method argument is trailing and optional, so existing positional JS calls keep their meaning and their results.

### Added

- **`WasmUrbanSegment`: all three access-point delay sources of Equation 18-7.** Nine trailing constructor arguments (`access_point_delays_s`, `n_influential_access_points`, `pct_left_turns_access`, `pct_right_turns_access`, `access_left_bay_adequate`, `access_right_bay_adequate`, `midsegment_other_delay_s`, `analysis_period_h`, `access_point_turn_delay_speed_mph`) plus an `add_access_point()` method taking one Chapter 30 Section 4 approach per call as a serde object. The binding previously forced the Exhibit 18-13 planning estimate with its built-in 10%/10% turn baseline, which put the Chapter 30 Example Problem 1 running time (33.54 s) and travel speed (23.67 mi/h) out of reach; both now reproduce, from the published Exhibit 30-35 per-point delays and from the Section 4 procedure computing them (0.193 and 0.194 s/veh, inside-lane blockage probability 0.115).
- **`WasmUrbanSegment` Step 2 intermediates**, as getters and in `results_to_js_value()`: the speed constant S_0 and the f_CS, f_A, and f_pk adjustments of Exhibit 18-11, the signal spacing factor f_L of Equation 18-4, the proximity factor f_v of Equation 18-6, and `access_point_delays_computed()` for the per-point left/right/total delay and blockage-probability breakdown of Exhibit 30-35.
- **`WasmUrbanFacility.add_segment`: the segment geometry the Chapter 18 free-flow speed depends on** — `upstream_intersection_width_ft`, `restrictive_median_length_ft`, `proportion_with_curb`, `proportion_on_street_parking`, `prop_opposing_left_accessible`, `signal_spacing_ft`, and `free_flow_speed_override_mph` — followed by the same nine access-point arguments as the segment constructor. A facility built from the Chapter 30 Example Problem 1 segment now reproduces the published base free-flow speed of 40.78 mi/h and travel speed of 23.67 mi/h instead of falling back to the Exhibit 18-5 defaults.
- **`WasmUrbanFacility.add_segment_from_config()`**, taking one segment as a configuration object in the serde schema of the library's `UrbanSegment` — the same shape as the library fixture files, so a fixture segment loads verbatim in one call instead of through the 31-argument positional `add_segment` (which stays).
- **`WasmUrbanFacility.add_segment_summary()` and `aggregate()`**, the Exhibit 16-7 "HCM method output" path. A segment can be given by its already-known length, base free-flow speed, travel speed, spatial stop rate, v/c ratio, and LOS letter, and `aggregate()` runs Chapter 16 Steps 1 through 4 over those measures without the Chapter 18 engine. This is how the published example problems are stated, and it makes Chapter 29 Example Problem 1 expressible (facility base free-flow speed 40.1 mi/h, LOS C, poorest segment LOS D). `analyze()` now throws rather than recompute a facility holding such a segment, because there are no Chapter 18 inputs behind one.
- **`WasmUrbanReliability`: monthly snowfall, the calendar, and the boundary-signal delay parameters.** Trailing constructor arguments `monthly_total_snowfall_in` (the fifth monthly weather array, taking 0, 1, or 12 entries like the others), `jan1_day_of_week`, and the facility `prop_left_turn_lanes`; trailing `add_segment` arguments `k_factor`, `i_factor`, and `approach_lanes` for the Exhibit 19-14 incremental delay factor, the upstream filtering factor, and Σ N_n of Equation 29-27. Snowfall was hard-coded to zero, which silently removed the strongest weather events from the scenario stream; with the Lincoln climatology of Chapter 29 Example Problem 4 supplied, the wasm run now matches the library's own run of that fixture.
- **`WasmUrbanReliability.add_atdm_strategy()`**, taking an `AtdmStrategy` serde object (every field defaults to no effect, so supply only what the strategy changes). This reaches the Chapter 17 Section 4 strategy, work zone, and special event hook, including Example Problem 5's reallocation of 5 s of split to the coordinated through phase and the Chapter 37 Section 5 adaptive signal control form.
- **`WasmUrbanReliability.num_oversaturated_scenarios()`**, the count of scenarios in which a boundary through movement ran over capacity. These are the scenarios that carry a residual queue into the next analysis period, so the count says how much of the travel-time distribution's tail comes from oversaturation rather than from weather or incidents.

HCM-coverage releases stay within the 0.3.x family so the CrossTraffic crates carry a matched version line.

## 0.3.2 — 2026-08

### Added

- **`WasmAlternativeIntersection`**: the HCM Ch.23 Part C RCUT/MUT operational analysis (Exhibit 23-47 Steps 6-10). The constructor takes a configuration object matching the library's `AlternativeIntersection` serde schema (form, movements with ordered junction journeys); results come back as per-movement Equation 23-60 ETT/LOS rows plus the Equation 23-61/23-62 approach and intersection aggregations, with intersection LOS from Exhibit 23-13.
- **Part C helper functions**, exported as plain functions: `edtt_merge` (Equation 23-58) and `edtt_stop_or_signal` (Equation 23-59) so the UI can compute extra distance travel time from crossover geometry, `uturn_saturation_adjustment` (Exhibit 23-52), `stop_junction_delay` (the Chapter 20 gap-acceptance capacity/delay/queue bundle used at STOP-controlled crossovers), and `dlt_offset` (the DLT supplemental-intersection offset, Equations 23-63 through 23-68). Together with the existing `WasmDisplacedLeftTurn` (Equation 23-69) this completes the wasm surface for the Part C forms; the library implementations are validated against Chapter 34 Example Problems 12-16.

HCM-coverage releases stay within the 0.3.x family so the CrossTraffic crates carry a matched version line.

## 0.3.1 — 2026-08

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
