use wasm_bindgen::prelude::*;
use transportations_library::hcm::chapter23::alternative_intersections as alt;
use transportations_library::hcm::chapter23::alternative_intersections::{
    AlternativeIntersection, Approach,
};
use transportations_library::hcm::chapter23::los_alternative_intersection_od;

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct WasmAlternativeIntersection {
    inner: AlternativeIntersection,
}

#[wasm_bindgen]
impl WasmAlternativeIntersection {

    /// Build an RCUT or MUT alternative-intersection analysis (HCM Ch.23
    /// Part C, Exhibit 23-47 Steps 6-10) from a configuration object
    /// matching the serde schema of
    /// `hcm::chapter23::alternative_intersections::AlternativeIntersection`:
    ///
    /// ```json
    /// {
    ///   "form": "RcutFourLeg",
    ///   "movements": [
    ///     {
    ///       "label": "NB L",
    ///       "approach": "Nb",
    ///       "demand_veh_h": 167.0,
    ///       "edtt_s": 15.9,
    ///       "junctions": [
    ///         { "type": "stop", "flow_veh_h": 167.0,
    ///           "conflicting_flow_veh_h": 1189.0,
    ///           "critical_headway_s": 4.4, "followup_headway_s": 2.6 },
    ///         { "type": "provided", "control_delay_s": 20.2 },
    ///         { "type": "merge" }
    ///       ]
    ///     }
    ///   ]
    /// }
    /// ```
    ///
    /// `form` is one of `RcutFourLeg` / `RcutThreeLeg` / `MutFourLeg` /
    /// `MutThreeLeg`. Each movement lists the junctions of its journey in
    /// traversal order (Exhibits 23-48 through 23-50): `provided` carries a
    /// Chapter 19 signalized control delay (optional `vc_gt_1` / `rq_gt_1`
    /// flags force LOS F), `stop` is evaluated here with the Chapter 20
    /// gap-acceptance procedure (optional `storage_ft` / `queue_spacing_ft`
    /// for the queue-storage check), and `merge` is a zero-delay free-flow
    /// merge. `edtt_s` is the Step 7 extra distance travel time (compute it
    /// with [`edtt_merge`] or [`edtt_stop_or_signal`]); `analysis_period_h`
    /// defaults to 0.25.
    #[wasm_bindgen(constructor)]
    pub fn new(config: JsValue) -> Result<WasmAlternativeIntersection, JsValue> {
        let inner: AlternativeIntersection = serde_wasm_bindgen::from_value(config)
            .map_err(|e| JsValue::from_str(&format!("invalid alternative-intersection configuration: {e}")))?;
        Ok(WasmAlternativeIntersection { inner })
    }

    /// Per-movement results as a JS array (label, per-junction control
    /// delays in journey order, total control delay, EDTT, ETT per Equation
    /// 23-60, v/c and queue-storage flags, LOS per Exhibit 23-13).
    pub fn movement_results_to_js_value(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.inner.evaluate()).unwrap_or(JsValue::NULL)
    }

    /// Demand-weighted approach experienced travel time ETT_A, s/veh (HCM
    /// Equation 23-61). `approach` is "EB", "WB", "NB", or "SB" (case-
    /// insensitive). `undefined` when the approach carries no demand.
    pub fn get_approach_ett_s(&self, approach: &str) -> Result<Option<f64>, JsValue> {
        let approach = match approach.to_ascii_lowercase().as_str() {
            "eb" => Approach::Eb,
            "wb" => Approach::Wb,
            "nb" => Approach::Nb,
            "sb" => Approach::Sb,
            other => {
                return Err(JsValue::from_str(&format!(
                    "unknown approach {other:?}: expected EB, WB, NB, or SB"
                )))
            }
        };
        Ok(self.inner.approach_ett(approach))
    }

    /// Demand-weighted intersection experienced travel time ETT_I, s/veh
    /// (HCM Equation 23-62). `undefined` when no movement carries demand.
    pub fn get_intersection_ett_s(&self) -> Option<f64> {
        self.inner.intersection_ett()
    }

    /// Intersection LOS letter from Exhibit 23-13 applied to ETT_I, e.g.
    /// "C". Per-movement LOS F forcing (v/c > 1 or queue storage exceeded)
    /// is reported in the movement results, not here.
    pub fn get_intersection_los(&self) -> Option<String> {
        self.inner
            .intersection_ett()
            .map(|ett| format!("{:?}", los_alternative_intersection_od(ett, false, false)))
    }
}

/// HCM Equation 23-58: extra distance travel time for a rerouted movement
/// at an RCUT with merges, s/veh.
///
/// * `dist_to_crossover_ft` / `dist_from_crossover_ft` — distances D_t and
///   D_f between the main junction and the U-turn crossover, ft.
/// * `free_flow_speed_mph` — major-street free-flow speed S_f, mi/h.
/// * `accel_decel_s` — deceleration/acceleration term `a`, s: 10 for a
///   minor-street left turn, 15 for a minor-street through movement.
#[wasm_bindgen]
pub fn edtt_merge(
    dist_to_crossover_ft: f64,
    dist_from_crossover_ft: f64,
    free_flow_speed_mph: f64,
    accel_decel_s: f64,
) -> f64 {
    alt::edtt_merge(
        dist_to_crossover_ft,
        dist_from_crossover_ft,
        free_flow_speed_mph,
        accel_decel_s,
    )
}

/// HCM Equation 23-59: extra distance travel time for a rerouted movement
/// at an RCUT or MUT with STOP signs or signals, s/veh. No acceleration/
/// deceleration term, because that delay is already captured by the STOP or
/// signal control-delay computation.
#[wasm_bindgen]
pub fn edtt_stop_or_signal(
    dist_to_crossover_ft: f64,
    dist_from_crossover_ft: f64,
    free_flow_speed_mph: f64,
) -> f64 {
    alt::edtt_stop_or_signal(
        dist_to_crossover_ft,
        dist_from_crossover_ft,
        free_flow_speed_mph,
    )
}

/// HCM Exhibit 23-52: saturation flow rate adjustment factor for a
/// signalized MUT/RCUT U-turn crossover, by median width (0.80 below 35 ft,
/// 0.85 through 80 ft, 0.95 above).
#[wasm_bindgen]
pub fn uturn_saturation_adjustment(median_width_ft: f64) -> f64 {
    alt::uturn_saturation_adjustment(median_width_ft)
}

/// Evaluate a STOP-controlled junction movement with the Chapter 20
/// gap-acceptance capacity (Equation 20-18) and control delay (Equation
/// 20-61). Returns `{ capacity_veh_h, vc_ratio, control_delay_s,
/// queue_95_veh }`. The default U-turn crossover headways are t_c = 4.4 s
/// and t_f = 2.6 s (Part C Step 5).
#[wasm_bindgen]
pub fn stop_junction_delay(
    flow_veh_h: f64,
    conflicting_flow_veh_h: f64,
    critical_headway_s: f64,
    followup_headway_s: f64,
    analysis_period_h: f64,
) -> JsValue {
    let result = alt::stop_junction_delay(
        flow_veh_h,
        conflicting_flow_veh_h,
        critical_headway_s,
        followup_headway_s,
        analysis_period_h,
    );
    serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
}

/// HCM Equations 23-63 through 23-68: the DLT supplemental-intersection
/// offset so displaced left-turn vehicles arrive during the guaranteed
/// green window at the main intersection. Returns `{ tt_dlt_s, st_dlt_s,
/// st_th_s, offset_supp_s }` with the adjusted offset wrapped into
/// `[0, C)`.
///
/// * `td_dlt_ft` — displaced left-turn roadway travel distance TD_DLT, ft.
/// * `sf_dlt_mph` — displaced left-turn roadway free-flow speed, mi/h.
/// * `lag_dlt_s` / `lag_th_s` — durations from the reference point to the
///   start of the DLT phase (supplemental) and the major-street through
///   phase (main), s.
/// * `offset_supp_s` / `offset_main_s` — initial offsets, s.
/// * `cycle_s` — background cycle length C, s.
#[wasm_bindgen]
pub fn dlt_offset(
    td_dlt_ft: f64,
    sf_dlt_mph: f64,
    lag_dlt_s: f64,
    lag_th_s: f64,
    offset_supp_s: f64,
    offset_main_s: f64,
    cycle_s: f64,
) -> JsValue {
    let result = alt::dlt_offset(
        td_dlt_ft,
        sf_dlt_mph,
        lag_dlt_s,
        lag_th_s,
        offset_supp_s,
        offset_main_s,
        cycle_s,
    );
    serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
}
