use transportations_library::basicfreeways::composite_grade::CompositeGrade;
use transportations_library::basicfreeways::mixed_flow::MixedFlowSegment;
use wasm_bindgen::prelude::*;

/// Message text a failed analysis carries across to JS.
///
/// Split out as a plain string function because this crate's tests run on the host, where a
/// `JsValue` cannot be constructed at all. What is worth testing here is not that an error is
/// returned but that the core's message survives: the mixed-flow truck curves are digitised only
/// for the grades and speeds the two published worked examples need, and everything outside that
/// is refused with a message naming the exhibit that would have to be digitised. A wrapper that
/// replaced or truncated that message would turn a precise refusal into "analysis failed".
fn analysis_error(what: &str, e: &str) -> String {
    format!("{what}: {e}")
}

/// Single-grade mixed-flow analysis (HCM Chapter 26, Equations 26-1 through 26-22).
///
/// This is the alternative to the passenger-car-equivalent method that [`WasmBasicFreeways`]
/// runs. Chapter 12 converts trucks into passenger cars and analyses one homogeneous stream,
/// which stops describing anything real on a sustained steep grade, where the trucks settle
/// towards a crawl speed the automobiles never approach. The mixed-flow model carries
/// automobiles, single-unit trucks and tractor-trailers as three populations with their own
/// travel time rates and combines them at the end. The two disagree on purpose: on the 5% grade
/// of Chapter 26 Example Problem 5 the PCE path gives 25.2 veh/mi/ln and this one gives 31.7.
///
/// [`WasmBasicFreeways`]: crate::middleware::corust::wasm_basicfreeways::WasmBasicFreeways
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct WasmMixedFlow {
    inner: MixedFlowSegment,
}

#[wasm_bindgen]
impl WasmMixedFlow {
    /// Build a single-grade mixed-flow analysis from a configuration object matching the serde
    /// schema of the library's `MixedFlowSegment` — the shape of the library's own fixtures, so
    /// the `tests/ExampleCases/hcm/Chapter26/ep5_mixed_flow.json` of Example Problem 5 passes
    /// verbatim:
    ///
    /// ```json
    /// {
    ///   "ffs": 65.0, "length": 2.0, "grade": 5.0,
    ///   "v_mix": 1500.0, "p_sut": 0.05, "p_tt": 0.10, "caf_ao": 1.0
    /// }
    /// ```
    ///
    /// Units are the manual's and are not interchangeable with the ones the rest of this crate
    /// uses. `length` is in MILES, not the feet a Chapter 15 subsegment takes. `grade` is in
    /// PERCENT, so a 5% upgrade is 5.0. `p_sut` and `p_tt` are DECIMALS, so 5% single-unit
    /// trucks is 0.05, which is the opposite convention from `grade` in the same object. A
    /// percent handed to `p_sut` is caught, since the proportions must sum below one, and a
    /// decimal handed to `grade` is caught, since the curves are digitised per tabulated grade
    /// and 0.05 is not one of them. A length in feet is the one that is not caught, because
    /// there is no upper bound on a grade's length to check it against.
    ///
    /// Only `caf_ao` is optional, defaulting to 1.0, which is the no-adjustment case. Every
    /// other field is required, and the core struct denies unknown keys, so a misspelled key is
    /// rejected here rather than silently defaulted — including a misspelled `caf_ao`, which
    /// before library 0.3.5's `deny_unknown_fields` would have been dropped and quietly analyzed
    /// unadjusted.
    ///
    /// The analysis itself is deferred to [`Self::results_to_js_value`], because the inputs can
    /// be well-formed and still lie outside the digitised truck curves, which is a refusal about
    /// coverage rather than about the configuration.
    #[wasm_bindgen(constructor)]
    pub fn new(config: JsValue) -> Result<WasmMixedFlow, JsValue> {
        let inner: MixedFlowSegment = serde_wasm_bindgen::from_value(config)
            .map_err(|e| JsValue::from_str(&format!("invalid mixed-flow configuration: {e}")))?;
        Ok(WasmMixedFlow { inner })
    }

    /// Full Chapter 26 chain as a JS object in the serde schema of `MixedFlowResult`: the
    /// Equation 26-1 through 26-5 capacity side (`caf_t_mix`, `rho_g_mix`, `caf_g_mix`,
    /// `caf_mix`, `capacity_ao`, `capacity_mix`), the Equation 26-11/26-12 kinematic truck rates
    /// and the Equation 26-13/26-14 free-flow side (`tau_sut_kin`, `tau_tt_kin`, `tau_a_ffs`,
    /// `tau_mix_ffs`, `ffs_mix`, `saf_mix`), the Equation 26-16 breakpoint with the auto-only one
    /// it is built from (`bp_ao`, `bp_mix`), the Equation 26-19 calibration speeds and the
    /// Equation 26-20 exponent (`s_calib_cap`, `s_calib_90cap`, `phi_mix`), and the Equation
    /// 26-21/26-22 answer (`s_mix`, `d_mix`, `oversaturated`).
    ///
    /// `s_mix` and `d_mix` are ABSENT (`undefined` in JS, not `null` — serde crosses `None` as
    /// `undefined`) when demand exceeds mixed-flow capacity, which Chapter 26 Step 2 calls LOS F
    /// and stops on rather than reporting a speed. A page guarding on `=== null` never fires;
    /// guard on `== null` or `typeof`.
    ///
    /// Throws when the grade or free-flow speed lands outside the digitised truck
    /// curves (length never throws — see the constructor note on lengths in feet). The message names the exhibit that would have to be digitised, because these
    /// curves are published as figures with no closed form anywhere in either chapter and each
    /// grade settles at its own crawl speed, so extrapolating between them would be quietly
    /// wrong rather than approximately right.
    pub fn results_to_js_value(&self) -> Result<JsValue, JsValue> {
        let result = self.analyze()?;
        serde_wasm_bindgen::to_value(&result)
            .map_err(|e| JsValue::from_str(&format!("could not serialize mixed-flow result: {e}")))
    }

    /// Equation 26-5 mixed-flow capacity C_mix, veh/h/ln. 1,725 in Example Problem 5, against
    /// the 2,350 pc/h/ln the same segment carries under auto-only conditions.
    pub fn get_capacity_mix(&self) -> Result<f64, JsValue> {
        Ok(self.analyze()?.capacity_mix)
    }

    /// Equation 26-22 mixed-flow density D_mix, veh/mi/ln, or undefined when demand exceeds
    /// mixed-flow capacity.
    pub fn get_density(&self) -> Result<Option<f64>, JsValue> {
        Ok(self.analyze()?.d_mix)
    }

    fn analyze(
        &self,
    ) -> Result<transportations_library::basicfreeways::mixed_flow::MixedFlowResult, JsValue> {
        self.inner
            .analyze()
            .map_err(|e| JsValue::from_str(&analysis_error("mixed-flow analysis failed", &e)))
    }
}

/// Composite-grade mixed-flow analysis (HCM Chapter 25, Equations 25-53 through 25-70).
///
/// Chapter 25 states that its equations are the Chapter 26 ones under different numbers, and the
/// thing it adds is chaining: a truck enters each grade at the speed the grade above it left it
/// at, rather than at free-flow speed. That is the whole point of the surface. Analysing the
/// three grades of Example Problem 11 independently and averaging them would report a facility
/// that is optimistic on every segment, with nothing failing anywhere.
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct WasmCompositeGrade {
    inner: CompositeGrade,
}

#[wasm_bindgen]
impl WasmCompositeGrade {
    /// Build a composite-grade analysis from a configuration object matching the serde schema of
    /// the library's `CompositeGrade` — the shape of the library's own fixtures, so the
    /// `tests/ExampleCases/hcm/Chapter25/ep11_composite_grade.json` of Example Problem 11 passes
    /// verbatim:
    ///
    /// ```json
    /// {
    ///   "ffs": 65.0, "v_mix": 1500.0, "p_sut": 0.05, "p_tt": 0.10, "caf_ao": 1.0,
    ///   "segments": [
    ///     { "length": 1.5, "grade": 3.0 },
    ///     { "length": 2.0, "grade": 2.0 },
    ///     { "length": 1.0, "grade": 5.0 }
    ///   ]
    /// }
    /// ```
    ///
    /// `segments` is in the order a vehicle meets the grades, and the order is an input rather
    /// than a presentation detail: reversing the three grades of Example Problem 11 puts the 5%
    /// first and slows the trucks to speeds no digitised curve covers, which is refused by name.
    /// Units follow the single-grade surface, `length` in miles and `grade` in percent, with the
    /// truck proportions as decimals and shared across all segments.
    ///
    /// This constructor adds no validation of its own, which is worth saying because the rest of
    /// this crate's config-object bindings do. `segments` has no serde default, so an omitted or
    /// misspelled key is rejected by deserialization rather than deserializing into an empty
    /// facility, and an explicitly empty list is rejected by the core's own `validate` before it
    /// reaches the per-segment capacity minimum that would panic on it. The only serde-defaulted
    /// field on either mixed-flow surface is `caf_ao`, whose default of 1.0 is the
    /// no-adjustment case rather than a stand-in for something the caller meant to supply, and
    /// since library 0.3.5 both input structs deny unknown keys, so even a misspelled `caf_ao`
    /// is rejected naming the key rather than dropped. So there is no input here that arrives
    /// wrong and still produces a finished answer, which is the condition the guards elsewhere
    /// in this crate exist for.
    #[wasm_bindgen(constructor)]
    pub fn new(config: JsValue) -> Result<WasmCompositeGrade, JsValue> {
        let inner: CompositeGrade = serde_wasm_bindgen::from_value(config).map_err(|e| {
            JsValue::from_str(&format!("invalid composite-grade configuration: {e}"))
        })?;
        Ok(WasmCompositeGrade { inner })
    }

    /// Full Chapter 25 chain as a JS object in the serde schema of `CompositeGradeResult`: a
    /// `segments` array carrying each grade's capacity side (`caf_g_mix`, `caf_mix`,
    /// `capacity_mix`), the rates the chaining is built from (`tau_f_sut_kin`, `tau_f_tt_kin`,
    /// `tau_s_sut_kin`, `tau_s_tt_kin`, `tau_f_a`, `tau_s_a`, `decelerating`), and the segment
    /// answer (`tau_mix`, `s_mix`, `travel_time`, `spot_speeds`, `space_speeds`); then the
    /// governing `capacity_mix` with the `governing_segment` index that sets it, the
    /// `entry_spot_speeds` at the facility entry, `total_length`, `total_travel_time`, the
    /// Equation 25-70 `s_mix_overall`, `overall_space_speeds`, and `oversaturated`.
    ///
    /// `spot_speeds`, `space_speeds` and `overall_space_speeds` are `[automobiles, SUTs, TTs]`.
    ///
    /// Throws when the chain reaches a grade or an entry speed outside the digitised truck
    /// curves, naming what is missing. This is more reachable here than on the single-grade
    /// surface, because the entry speed into each segment is computed rather than given, so a
    /// configuration whose every field is inside the digitised range can still walk out of it.
    pub fn results_to_js_value(&self) -> Result<JsValue, JsValue> {
        let result = self.analyze()?;
        serde_wasm_bindgen::to_value(&result).map_err(|e| {
            JsValue::from_str(&format!("could not serialize composite-grade result: {e}"))
        })
    }

    /// Number of grades this configuration deserialized into — a serialization sanity check,
    /// not a defense against a caller who built the config short: this only echoes what
    /// deserialized, which equals the caller's own `segments.length`. A composite grade that
    /// arrives short a segment is a valid facility and is not detectable at this layer.
    pub fn get_segment_count(&self) -> u32 {
        self.inner.segments.len() as u32
    }

    /// Governing mixed-flow capacity, veh/h/ln — the tightest of the per-segment capacities,
    /// 1,746 in Example Problem 11, set by the 1 mi 5% grade.
    pub fn get_capacity_mix(&self) -> Result<f64, JsValue> {
        Ok(self.analyze()?.capacity_mix)
    }

    /// Equation 25-70 overall mixed-flow speed S_mix,oa, mi/h: the total length over the summed
    /// segment travel times, 55.6 in Example Problem 11.
    pub fn get_overall_speed(&self) -> Result<f64, JsValue> {
        Ok(self.analyze()?.s_mix_overall)
    }

    fn analyze(
        &self,
    ) -> Result<
        transportations_library::basicfreeways::composite_grade::CompositeGradeResult,
        JsValue,
    > {
        self.inner.analyze().map_err(|e| {
            JsValue::from_str(&analysis_error("composite-grade analysis failed", &e))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A grade and free-flow speed combination Stage 1 never digitised must reach the caller
    /// naming the exhibit, not as a bare failure. FFS 70 with everything else at the Example
    /// Problem 5 values is inside every validation the core performs and outside its curves,
    /// which is exactly the case a calculator page would produce by moving one field.
    #[test]
    fn single_grade_refusal_names_the_missing_exhibit() {
        let seg = MixedFlowSegment {
            ffs: 70.0,
            length: 2.0,
            grade: 5.0,
            v_mix: 1500.0,
            p_sut: 0.05,
            p_tt: 0.10,
            caf_ao: 1.0,
        };
        let e = seg.analyze().expect_err("FFS 70 is outside Stage 1");
        let msg = analysis_error("mixed-flow analysis failed", &e);
        assert!(
            msg.contains("Chapter 26 Appendix A"),
            "the exhibit that would have to be digitised must survive the wrapper: {msg}"
        );
        assert!(msg.contains("70 mi/h FFS"), "{msg}");
        assert!(msg.starts_with("mixed-flow analysis failed: "), "{msg}");
    }

    /// The composite surface reaches the spot curves first, so its refusal names the spot
    /// exhibits rather than the travel time ones.
    #[test]
    fn composite_refusal_names_the_missing_exhibit() {
        let mut c = ep11();
        c.segments[1].grade = 7.0;
        let e = c.analyze().expect_err("7% is outside Stage 1");
        let msg = analysis_error("composite-grade analysis failed", &e);
        assert!(
            msg.contains("Exhibit 25-20/25-21"),
            "the exhibit that would have to be digitised must survive the wrapper: {msg}"
        );
        assert!(msg.contains("digitised"), "{msg}");
    }

    /// Why the composite constructor carries no emptiness guard. The core's `validate` rejects an
    /// empty facility before the per-segment capacity minimum, which would panic on it, and a
    /// panic in wasm aborts the module rather than throwing something a caller can catch. If that
    /// check is ever reordered behind the minimum this test fails on the unwind, and the guard
    /// belongs back in the constructor.
    #[test]
    fn an_empty_composite_grade_is_refused_by_the_core_not_panicked_on() {
        let mut c = ep11();
        c.segments.clear();
        let e = std::panic::catch_unwind(move || c.analyze())
            .expect("an empty facility must not panic inside the core")
            .expect_err("an empty facility is not analyzable");
        assert!(e.contains("at least one segment"), "{e}");
    }

    fn ep11() -> CompositeGrade {
        use transportations_library::basicfreeways::composite_grade::GradeSegment;
        CompositeGrade {
            ffs: 65.0,
            v_mix: 1500.0,
            p_sut: 0.05,
            p_tt: 0.10,
            caf_ao: 1.0,
            segments: vec![
                GradeSegment { length: 1.5, grade: 3.0 },
                GradeSegment { length: 2.0, grade: 2.0 },
                GradeSegment { length: 1.0, grade: 5.0 },
            ],
        }
    }
}

