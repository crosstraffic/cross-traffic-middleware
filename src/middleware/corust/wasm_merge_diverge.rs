use wasm_bindgen::prelude::*;
use transportations_library::common::HcmVersion;
use transportations_library::merge_diverge as md;
use transportations_library::merge_diverge::{
    AdjacentRampType, RampLanes, RampSegment, RampSide, RampType, ServiceDemandBasis, TerrainType,
};

fn parse_adjacent(s: &str) -> AdjacentRampType {
    match s.to_lowercase().as_str() {
        "on_ramp" | "onramp" | "on-ramp" | "on" => AdjacentRampType::OnRamp,
        "off_ramp" | "offramp" | "off-ramp" | "off" => AdjacentRampType::OffRamp,
        _ => AdjacentRampType::None,
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct WasmRampSegment {
    inner: RampSegment,
}

impl WasmRampSegment {
    /// The stepwise methods implement the 7th Edition's numbered steps (lane distribution,
    /// v_12, Exhibit 14-13 speeds), which Edition 7.1 replaced with a different structure.
    /// They throw on a "7.1" segment rather than silently returning 7th Edition numbers.
    fn require_v7(&self, method: &str) -> Result<(), JsValue> {
        if self.inner.version == HcmVersion::V7_1 {
            return Err(JsValue::from_str(&format!(
                "{method}() implements the 7th Edition step structure, but this segment is \
                 version \"7.1\". Use run_analysis() and analysis_v7_1() instead."
            )));
        }
        Ok(())
    }
}

#[wasm_bindgen]
impl WasmRampSegment {

    #[wasm_bindgen(constructor)]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        ramp_type: Option<String>,
        ramp_side: Option<String>,
        ramp_lanes: Option<u32>,
        freeway_lanes: Option<u32>,
        freeway_ffs: Option<f64>,
        ramp_ffs: Option<f64>,
        accel_lane_length: Option<f64>,
        accel_lane_length2: Option<f64>,
        decel_lane_length: Option<f64>,
        decel_lane_length2: Option<f64>,
        freeway_demand: Option<f64>,
        ramp_demand: Option<f64>,
        phf: Option<f64>,
        heavy_vehicle_pct: Option<f64>,
        ramp_heavy_vehicle_pct: Option<f64>,
        terrain: Option<String>,
        adjacent_upstream: Option<String>,
        upstream_distance: Option<f64>,
        upstream_ramp_flow: Option<f64>,
        adjacent_downstream: Option<String>,
        downstream_distance: Option<f64>,
        downstream_ramp_flow: Option<f64>,
        caf: Option<f64>,
        saf: Option<f64>,
        // HCM edition, "7" (default) or "7.1". Appended last so existing positional callers
        // are unchanged.
        version: Option<String>,
    ) -> Self {
        let mut inner = RampSegment::new();

        if let Some(rt) = ramp_type {
            inner.ramp_type = match rt.to_lowercase().as_str() {
                "off_ramp" | "offramp" | "off-ramp" | "diverge" => RampType::OffRamp,
                "major_merge" => RampType::MajorMerge,
                "major_diverge" => RampType::MajorDiverge,
                _ => RampType::OnRamp,
            };
        }
        if let Some(rs) = ramp_side {
            inner.ramp_side = match rs.to_lowercase().as_str() {
                "left" => RampSide::Left,
                _ => RampSide::Right,
            };
        }
        if let Some(rl) = ramp_lanes {
            inner.ramp_lanes = match rl {
                2 => RampLanes::TwoLane,
                _ => RampLanes::OneLane,
            };
        }
        if let Some(t) = terrain {
            inner.terrain = match t.to_lowercase().as_str() {
                "rolling" => TerrainType::Rolling,
                "mountainous" => TerrainType::Mountainous,
                _ => TerrainType::Level,
            };
        }
        if let Some(a) = adjacent_upstream {
            inner.adjacent_upstream = parse_adjacent(&a);
        }
        if let Some(a) = adjacent_downstream {
            inner.adjacent_downstream = parse_adjacent(&a);
        }
        if let Some(v) = freeway_lanes {
            inner.freeway_lanes = v;
        }
        if let Some(v) = freeway_ffs {
            inner.freeway_ffs = v;
        }
        if let Some(v) = ramp_ffs {
            inner.ramp_ffs = v;
        }
        if accel_lane_length.is_some() {
            inner.accel_lane_length = accel_lane_length;
        }
        if accel_lane_length2.is_some() {
            inner.accel_lane_length2 = accel_lane_length2;
        }
        if decel_lane_length.is_some() {
            inner.decel_lane_length = decel_lane_length;
        }
        if decel_lane_length2.is_some() {
            inner.decel_lane_length2 = decel_lane_length2;
        }
        if let Some(v) = freeway_demand {
            inner.freeway_demand = v;
        }
        if let Some(v) = ramp_demand {
            inner.ramp_demand = v;
        }
        if let Some(v) = phf {
            inner.phf = v;
        }
        if let Some(v) = heavy_vehicle_pct {
            inner.heavy_vehicle_pct = v;
        }
        if ramp_heavy_vehicle_pct.is_some() {
            inner.ramp_heavy_vehicle_pct = ramp_heavy_vehicle_pct;
        }
        if upstream_distance.is_some() {
            inner.upstream_distance = upstream_distance;
        }
        if upstream_ramp_flow.is_some() {
            inner.upstream_ramp_flow = upstream_ramp_flow;
        }
        if downstream_distance.is_some() {
            inner.downstream_distance = downstream_distance;
        }
        if downstream_ramp_flow.is_some() {
            inner.downstream_ramp_flow = downstream_ramp_flow;
        }
        if let Some(v) = caf {
            inner.caf = v;
        }
        if let Some(v) = saf {
            inner.saf = v;
        }
        if let Some(v) = version {
            // Unknown strings fall back to the 7th Edition default rather than trapping the
            // whole constructor; use the `version` setter for validated assignment.
            if let Ok(parsed) = v.parse::<HcmVersion>() {
                inner.version = parsed;
            }
        }

        WasmRampSegment { inner }
    }

    /// The HCM edition this segment analyzes under, "7" or "7.1".
    #[wasm_bindgen(getter)]
    pub fn version(&self) -> String {
        self.inner.version.to_string()
    }

    /// Set the HCM edition, "7" or "7.1". Throws on anything else.
    #[wasm_bindgen(setter)]
    pub fn set_version(&mut self, version: String) -> Result<(), JsValue> {
        self.inner.version = version
            .parse::<HcmVersion>()
            .map_err(|e| JsValue::from_str(&e))?;
        Ok(())
    }

    /// Full Edition 7.1 analysis as a JS object (null until `run_analysis()` has run on a
    /// version "7.1" segment). Fields follow the Rust `RampAnalysis` struct: flows, ffs_adj,
    /// speed_basic, speed_impedance, speed_avg (null far past capacity), capacity_per_lane,
    /// dc_ratio, the capacity checks, density, and los.
    pub fn analysis_v7_1(&self) -> Result<JsValue, JsValue> {
        match &self.inner.analysis_v7_1 {
            None => Ok(JsValue::NULL),
            Some(a) => serde_wasm_bindgen::to_value(a)
                .map_err(|e| JsValue::from_str(&format!("serialize error: {e}"))),
        }
    }

    /// Run the full HCM Ch.14 analysis (Steps 1-5) and return the LOS letter.
    /// Populates flows, v_12, capacities, density, and speeds.
    ///
    /// Returns `undefined` for a major merge operating under capacity, where the HCM defines no
    /// level of service and only the capacity checks apply. Callers must render that case rather
    /// than printing an empty letter.
    pub fn run_analysis(&mut self) -> Option<String> {
        self.inner.run_analysis().map(|los| {
            let c: char = los.into();
            c.to_string()
        })
    }

    /// Step 1: demand flows [v_F, v_R] in pc/h - Eq. 14-1.
    /// 7th Edition only; throws on a "7.1" segment.
    pub fn determine_demand_flow(&mut self) -> Result<Vec<f64>, JsValue> {
        self.require_v7("determine_demand_flow")?;
        let (v_f, v_r) = self.inner.determine_demand_flow();
        Ok(vec![v_f, v_r])
    }

    /// Step 2: flow in Lanes 1 and 2, v_12 (pc/h) - Eqs. 14-2..14-19.
    /// 7th Edition only; throws on a "7.1" segment.
    pub fn estimate_v12(&mut self) -> Result<f64, JsValue> {
        self.require_v7("estimate_v12")?;
        Ok(self.inner.estimate_v12())
    }

    /// Step 3: adjusted freeway capacity (pc/h) and capacity checks
    /// (Exhibits 14-10/14-12, Eq. 14-21).
    /// 7th Edition only; throws on a "7.1" segment.
    pub fn determine_capacity(&mut self) -> Result<f64, JsValue> {
        self.require_v7("determine_capacity")?;
        Ok(self.inner.determine_capacity())
    }

    /// Step 4: density in the ramp influence area (pc/mi/ln)
    /// - Eqs. 14-22/14-23/14-28.
    /// 7th Edition only; throws on a "7.1" segment.
    pub fn determine_density(&mut self) -> Result<f64, JsValue> {
        self.require_v7("determine_density")?;
        Ok(self.inner.determine_density())
    }

    /// Level of service letter - Exhibit 14-3.
    /// 7th Edition only; throws on a "7.1" segment.
    ///
    /// Returns `undefined` for a major merge under capacity; see [`Self::run_analysis`].
    pub fn determine_los(&mut self) -> Result<Option<String>, JsValue> {
        self.require_v7("determine_los")?;
        Ok(self.inner.determine_los().map(|los| {
            let c: char = los.into();
            c.to_string()
        }))
    }

    /// Step 5: speeds [S_R, S_O, S] in mi/h - Exhibits 14-13/14-14/14-15.
    /// S_O is NaN when the outer-lane speed does not apply.
    /// 7th Edition only; throws on a "7.1" segment.
    pub fn estimate_speed(&mut self) -> Result<Vec<f64>, JsValue> {
        self.require_v7("estimate_speed")?;
        let (s_r, s_o, s) = self.inner.estimate_speed();
        Ok(vec![s_r, s_o.unwrap_or(f64::NAN), s])
    }

    pub fn get_flow_freeway(&self) -> f64 {
        self.inner.get_flow_freeway()
    }

    pub fn get_flow_ramp(&self) -> f64 {
        self.inner.get_flow_ramp()
    }

    pub fn get_p_f(&self) -> Option<f64> {
        self.inner.p_f
    }

    pub fn get_v12(&self) -> f64 {
        self.inner.get_v12()
    }

    pub fn get_vr12(&self) -> f64 {
        self.inner.get_vr12()
    }

    pub fn get_capacity_freeway(&self) -> f64 {
        self.inner.get_capacity_freeway()
    }

    pub fn get_capacity_ramp(&self) -> f64 {
        self.inner.get_capacity_ramp()
    }

    pub fn get_vc_ratio(&self) -> f64 {
        self.inner.get_vc_ratio()
    }

    pub fn get_demand_exceeds_capacity(&self) -> Option<bool> {
        self.inner.demand_exceeds_capacity
    }

    pub fn get_exceeds_max_desirable(&self) -> Option<bool> {
        self.inner.exceeds_max_desirable
    }

    pub fn get_density(&self) -> f64 {
        self.inner.get_density()
    }

    pub fn get_speed_ramp(&self) -> f64 {
        self.inner.get_speed_ramp()
    }

    pub fn get_speed_outer(&self) -> Option<f64> {
        self.inner.get_speed_outer()
    }

    pub fn get_speed_avg(&self) -> f64 {
        self.inner.get_speed_avg()
    }

    pub fn get_los(&self) -> Option<String> {
        self.inner.get_los().map(|l| {
            let c: char = l.into();
            c.to_string()
        })
    }

    pub fn results_to_js_value(&self) -> JsValue {
        let obj = js_sys::Object::new();
        js_sys::Reflect::set(&obj, &JsValue::from_str("flow_freeway"), &JsValue::from(self.get_flow_freeway())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("flow_ramp"), &JsValue::from(self.get_flow_ramp())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("v12"), &JsValue::from(self.get_v12())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("vr12"), &JsValue::from(self.get_vr12())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("capacity_freeway"), &JsValue::from(self.get_capacity_freeway())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("capacity_ramp"), &JsValue::from(self.get_capacity_ramp())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("vc_ratio"), &JsValue::from(self.get_vc_ratio())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("density"), &JsValue::from(self.get_density())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("speed_ramp"), &JsValue::from(self.get_speed_ramp())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("speed_avg"), &JsValue::from(self.get_speed_avg())).unwrap();

        JsValue::from(obj)
    }
}

// =============================================================================
// Service flow rates and service volumes (HCM Chapter 28, Example Problem 5)
// =============================================================================

/// Reject a basis or a target density that would make the search return a
/// number rather than fail.
///
/// The core searches by comparing densities, and every comparison against a NaN
/// is false, so a non-finite input walks straight through the search's own
/// escape hatches. The "already past the target" test does not fire, so the
/// unachievable-LOS `None` is not returned; the bracket never doubles; and the
/// bisection drives its upper bound down to zero. What comes back is
/// `Some(4.4e-13)`, a service flow rate of zero rather than a failure, and the
/// caller has no way to tell it from a location whose LOS A threshold really is
/// that low. A NaN can arrive from an emptied number input as easily as from
/// arithmetic, so the check belongs at the boundary the values cross.
fn validate_search(basis: &ServiceDemandBasis, target_density: f64) -> Result<(), String> {
    if !target_density.is_finite() || target_density <= 0.0 {
        return Err(format!(
            "target_density must be a positive finite density in pc/mi/ln, got {target_density}"
        ));
    }
    let (name, value) = match *basis {
        ServiceDemandBasis::ApproachingFreeway { ramp_fraction } => {
            ("ramp_fraction", ramp_fraction)
        }
        ServiceDemandBasis::FixedFreeway { v_f } => ("v_f", v_f),
    };
    if !value.is_finite() || value < 0.0 {
        return Err(format!(
            "{name} must be a non-negative finite number, got {value}"
        ));
    }
    Ok(())
}

/// Service flow rate under ideal conditions SFI (pc/h) at a target ramp-influence
/// density - HCM Chapter 28, Example Problem 5.
///
/// Holds `template`'s geometry fixed and searches, under equivalent ideal
/// conditions (PHF = 1, no heavy vehicles, CAF = SAF = 1), for the demand that
/// drives the Equation 14-22 ramp-influence density to `target_density`. Use the
/// Exhibit 14-3 LOS thresholds 10, 20, 28, and 35 pc/mi/ln for LOS A through D.
/// The template's own demands, PHF, and heavy-vehicle percentages are ignored,
/// since the search supplies them; everything else about the segment matters.
///
/// `basis` is the serde form of the core's `ServiceDemandBasis`, an object
/// carrying the variant name as its single key:
///
/// ```json
/// { "ApproachingFreeway": { "ramp_fraction": 0.10 } }
/// { "FixedFreeway": { "v_f": 4896.0 } }
/// ```
///
/// The two are different questions and the returned quantity differs with them.
/// `ApproachingFreeway` varies the approaching freeway demand with the ramp
/// tracking it at `ramp_fraction * v_F` and returns v_F (Exhibit 28-4, Case 1);
/// `FixedFreeway` holds the approaching freeway at `v_f` pc/h ideal and varies
/// the ramp, returning v_R (Exhibit 28-5, Case 2). An unknown variant name, a
/// misspelled inner field (neither field has a serde default, so it arrives as a
/// missing field), and an object carrying both variant keys at once are all
/// rejected rather than resolved. What nothing can catch is a caller that sends
/// one basis meaning the other, since both return a plausible flow and only the
/// label on it changes.
///
/// Returns `undefined` when the target density is already exceeded at zero
/// varied demand, which is how the book reports an unachievable LOS (Exhibit
/// 28-5 prints NA for LOS A and B). That is a real answer about the location and
/// not a failure, so it is an absent value rather than a throw. Note that
/// `serde_wasm_bindgen` crosses `None` as `undefined` rather than `null`, so a
/// caller guarding on `=== null` never fires.
///
/// LOS E is not reachable through this function. It is a capacity limit rather
/// than a density, so it comes from `get_capacity_freeway()` and
/// `get_capacity_ramp()` on a segment run under the same ideal conditions.
#[wasm_bindgen]
pub fn ramp_service_flow_rate_ideal(
    template: &WasmRampSegment,
    basis: JsValue,
    target_density: f64,
) -> Result<Option<f64>, JsValue> {
    let basis: ServiceDemandBasis = serde_wasm_bindgen::from_value(basis).map_err(|e| {
        JsValue::from_str(&format!(
            "invalid service demand basis: {e} (expected {{\"ApproachingFreeway\": \
             {{\"ramp_fraction\": ...}}}} or {{\"FixedFreeway\": {{\"v_f\": ...}}}})"
        ))
    })?;
    validate_search(&basis, target_density).map_err(|e| JsValue::from_str(&e))?;
    Ok(md::ramp_service_flow_rate_ideal(
        &template.inner,
        &basis,
        target_density,
    ))
}

/// Convert an ideal-conditions service flow rate to the prevailing-condition
/// service flow rate and service volume - HCM Chapter 28, Example Problem 5.
///
/// SF = SFI x f_HV x f_p and SV = SF x PHF, so the returned `{ sf, sv }` are
/// both in veh/h while `sfi` is in pc/h. The two come back named rather than as
/// a pair because they are the same magnitude to within the PHF and a
/// transposed pair would read as a finished answer.
///
/// * `f_hv` - heavy-vehicle adjustment factor.
/// * `f_p` - driver-population factor, 1.0 for regular commuters.
/// * `phf` - peak hour factor.
#[wasm_bindgen]
pub fn ramp_service_volumes(sfi: f64, f_hv: f64, f_p: f64, phf: f64) -> JsValue {
    let (sf, sv) = md::ramp_service_volumes(sfi, f_hv, f_p, phf);
    let obj = js_sys::Object::new();
    js_sys::Reflect::set(&obj, &JsValue::from_str("sf"), &JsValue::from(sf)).unwrap();
    js_sys::Reflect::set(&obj, &JsValue::from_str("sv"), &JsValue::from(sv)).unwrap();
    JsValue::from(obj)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The guard exists for the answer a non-finite input produces, not for the
    /// input itself, so the test asserts what the core would have returned. A
    /// NaN target and a NaN ramp fraction both collapse to zero on a segment
    /// whose real LOS C service flow rate is above 5,000 pc/h, and neither
    /// returns the `None` that means the LOS is unachievable.
    #[test]
    fn non_finite_search_inputs_would_return_a_zero_service_flow_rate() {
        let template = RampSegment {
            ramp_type: RampType::OnRamp,
            ramp_side: RampSide::Right,
            ramp_lanes: RampLanes::OneLane,
            freeway_lanes: 3,
            freeway_ffs: 70.0,
            ramp_ffs: 40.0,
            accel_lane_length: Some(1000.0),
            terrain: TerrainType::Level,
            adjacent_upstream: AdjacentRampType::None,
            adjacent_downstream: AdjacentRampType::None,
            ..Default::default()
        };
        let good = ServiceDemandBasis::ApproachingFreeway { ramp_fraction: 0.10 };
        let nan_basis = ServiceDemandBasis::ApproachingFreeway {
            ramp_fraction: f64::NAN,
        };

        let nan_target = md::ramp_service_flow_rate_ideal(&template, &good, f64::NAN).unwrap();
        let nan_fraction = md::ramp_service_flow_rate_ideal(&template, &nan_basis, 28.0).unwrap();
        assert!(
            nan_target < 1e-9 && nan_fraction < 1e-9,
            "both must reach the caller as a service flow rate rather than as a failure, \
             got {nan_target} and {nan_fraction}"
        );

        assert!(validate_search(&good, f64::NAN).is_err());
        assert!(validate_search(&good, f64::INFINITY).is_err());
        assert!(validate_search(&good, 0.0).is_err());
        assert!(validate_search(&good, -28.0).is_err());
        assert!(validate_search(&nan_basis, 28.0).is_err());
        let nan_v_f = ServiceDemandBasis::FixedFreeway { v_f: f64::NAN };
        assert!(validate_search(&nan_v_f, 28.0).is_err());
        assert!(validate_search(&ServiceDemandBasis::FixedFreeway { v_f: -1.0 }, 28.0).is_err());

        // The control: the search the guard is wrapped around still runs, and
        // on this segment it finds the Exhibit 28-4 LOS C value.
        validate_search(&good, 28.0).expect("a real search must pass the guard");
        let sfi_c = md::ramp_service_flow_rate_ideal(&template, &good, 28.0).unwrap();
        assert!(
            (sfi_c - 5280.0).abs() < 6.0,
            "LOS C v_F should be the published 5,280 pc/h within the core's tolerance, got {sfi_c}"
        );
    }

    /// The message names the field that arrived wrong, because both variants
    /// carry exactly one number and a caller reading only "invalid" cannot tell
    /// which basis it sent.
    #[test]
    fn the_rejection_names_the_offending_field() {
        let e = validate_search(&ServiceDemandBasis::FixedFreeway { v_f: f64::NAN }, 28.0)
            .unwrap_err();
        assert!(e.contains("v_f"), "{e}");
        let e = validate_search(
            &ServiceDemandBasis::ApproachingFreeway {
                ramp_fraction: f64::NAN,
            },
            28.0,
        )
        .unwrap_err();
        assert!(e.contains("ramp_fraction"), "{e}");
        let e = validate_search(
            &ServiceDemandBasis::ApproachingFreeway { ramp_fraction: 0.1 },
            f64::NAN,
        )
        .unwrap_err();
        assert!(e.contains("target_density"), "{e}");
    }
}
