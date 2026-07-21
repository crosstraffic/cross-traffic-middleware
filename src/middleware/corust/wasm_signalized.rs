use wasm_bindgen::prelude::*;
use transportations_library::signalized::{
    LeftTurnMode, LeftTurnSequence, PhaseTiming, SignalApproach, SignalizedIntersection,
};
use transportations_library::hcm::common::intersection::{ControlType, Direction};

fn phase(phase_no: u8, duration_s: f64, yellow_s: f64, red_clearance_s: f64) -> PhaseTiming {
    PhaseTiming {
        phase_no,
        duration_s,
        yellow_s,
        red_clearance_s,
        max_green_s: None,
        passage_time_s: None,
        walk_s: None,
        ped_clear_s: None,
        min_green_s: None,
        detector_length_ft: None,
        recall_max: false,
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct WasmSignalizedIntersection {
    inner: SignalizedIntersection,
}

#[wasm_bindgen]
impl WasmSignalizedIntersection {

    /// Build a four-leg pretimed signalized intersection (HCM Ch.19).
    ///
    /// Array arguments are ordered NB, SB, EB, WB:
    /// * `volumes` — 12 values `[NB L, NB T, NB R, SB L, SB T, SB R, EB L, EB T, EB R, WB L, WB T, WB R]`, veh/h.
    /// * `lanes` — 12 values `[excl. left, through, excl. right]` per approach, same order.
    /// * `through_phase_s` — 4 through-phase durations D_p = G + Y + Rc, s.
    /// * `left_phase_s` — 4 protected left-turn phase durations, s (0 = no protected phase).
    ///
    /// A left turn with demand is treated as protected when it has an
    /// exclusive lane and a left phase duration, permitted otherwise (shared
    /// with the through lane when no exclusive left lane exists). A right
    /// turn without an exclusive lane shares the rightmost through lane.
    #[wasm_bindgen(constructor)]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        cycle_length_s: f64,
        analysis_period_h: Option<f64>,
        base_saturation_flow: Option<f64>,
        area_type_cbd: Option<bool>,
        peak_hour_factor: Option<f64>,
        volumes: Vec<f64>,
        lanes: Vec<u32>,
        through_phase_s: Vec<f64>,
        left_phase_s: Vec<f64>,
        yellow_s: Option<f64>,
        red_clearance_s: Option<f64>,
        pct_heavy_vehicles: Option<f64>,
        speed_limit_mph: Option<f64>,
        lane_width_ft: Option<f64>,
        ped_flow_ph: Option<f64>,
    ) -> Self {
        let dirs = [Direction::NB, Direction::SB, Direction::EB, Direction::WB];
        let through_phase_nos: [u8; 4] = [8, 4, 2, 6];
        let left_phase_nos: [u8; 4] = [3, 7, 5, 1];

        let mut volumes = volumes;
        volumes.resize(12, 0.0);
        let mut lanes = lanes;
        lanes.resize(12, 0);
        let mut through_phase_s = through_phase_s;
        through_phase_s.resize(4, 0.0);
        let mut left_phase_s = left_phase_s;
        left_phase_s.resize(4, 0.0);

        let y = yellow_s.unwrap_or(4.0);
        let rc = red_clearance_s.unwrap_or(0.0);
        let phv = pct_heavy_vehicles.unwrap_or(3.0);
        let spl = speed_limit_mph.unwrap_or(35.0);
        let lw = lane_width_ft.unwrap_or(12.0);
        let ped = ped_flow_ph.unwrap_or(0.0);
        let phf = match peak_hour_factor {
            Some(p) if p > 0.0 => Some(p),
            _ => None,
        };

        let mut approaches = Vec::with_capacity(4);
        for i in 0..4 {
            let (v_l, v_t, v_r) = (volumes[3 * i], volumes[3 * i + 1], volumes[3 * i + 2]);
            let excl_left = lanes[3 * i];
            let through = lanes[3 * i + 1].max(1);
            let excl_right = lanes[3 * i + 2];
            let shared_left = v_l > 0.0 && excl_left == 0;
            let shared_right = v_r > 0.0 && excl_right == 0;

            let (mode, sequence, left_phase) = if v_l <= 0.0 && excl_left == 0 {
                (LeftTurnMode::NotPresent, LeftTurnSequence::PermPerm, None)
            } else if left_phase_s[i] > 0.0 && excl_left > 0 {
                (
                    LeftTurnMode::Protected,
                    LeftTurnSequence::LeadLead,
                    Some(phase(left_phase_nos[i], left_phase_s[i], y, rc)),
                )
            } else {
                (LeftTurnMode::Permitted, LeftTurnSequence::PermPerm, None)
            };

            approaches.push(SignalApproach {
                direction: dirs[i],
                volume_left: v_l,
                volume_through: v_t,
                volume_right: v_r,
                volume_rtor: 0.0,
                peak_hour_factor: phf,
                pct_heavy_vehicles_left: phv,
                pct_heavy_vehicles_through: phv,
                platoon_ratio_left: 1.0,
                platoon_ratio_through: 1.0,
                upstream_filtering_i: 1.0,
                initial_queue_through_veh: 0.0,
                initial_queue_left_veh: 0.0,
                ped_flow_ph: ped,
                bike_flow_ph: 0.0,
                exclusive_left_lanes: excl_left,
                through_lanes: through,
                exclusive_right_lanes: excl_right,
                shared_left_through_lane: shared_left,
                shared_right_through_lane: shared_right,
                lane_width_ft: lw,
                grade_pct: 0.0,
                receiving_lanes: (through + u32::from(shared_right)).max(1),
                parking_present: false,
                parking_maneuvers_h: 0.0,
                bus_stops_h: 0.0,
                storage_left_ft: None,
                storage_through_ft: None,
                speed_limit_mph: spl,
                opposing_right_turn_influences_gaps: true,
                left_turn_mode: mode,
                left_turn_sequence: sequence,
                left_phase,
                through_phase: phase(through_phase_nos[i], through_phase_s[i], y, rc),
            });
        }

        let mut inner = SignalizedIntersection::new(cycle_length_s, approaches);
        if let Some(t) = analysis_period_h {
            inner.analysis_period_h = t;
        }
        if let Some(s) = base_saturation_flow {
            inner.base_saturation_flow = s;
        }
        if let Some(cbd) = area_type_cbd {
            inner.area_type_cbd = cbd;
        }
        inner.control = ControlType::PretimedSignal;
        WasmSignalizedIntersection { inner }
    }

    /// Build the intersection from a full configuration object matching the
    /// serde schema of `hcm::chapter19::signalized::SignalizedIntersection`
    /// (same shape as `tests/ExampleCases/hcm/Signalized/case1.json`).
    pub fn from_config(config: JsValue) -> Result<WasmSignalizedIntersection, JsValue> {
        let inner: SignalizedIntersection = serde_wasm_bindgen::from_value(config)
            .map_err(|e| JsValue::from_str(&format!("invalid intersection configuration: {e}")))?;
        Ok(WasmSignalizedIntersection { inner })
    }

    /// Run the full HCM Ch.19 motorized vehicle methodology (Steps 1-10 of
    /// Exhibit 19-18). Populates lane group, approach, and intersection results.
    pub fn analyze(&mut self) {
        self.inner.analyze();
    }

    pub fn get_cycle_length_s(&self) -> f64 {
        self.inner.get_cycle_length()
    }

    /// Intersection control delay d_I, s/veh (HCM Equation 19-29).
    pub fn get_intersection_delay_s(&self) -> Option<f64> {
        self.inner.get_intersection_delay()
    }

    /// Intersection LOS letter (HCM Exhibit 19-8), e.g. "D".
    pub fn get_intersection_los(&self) -> Option<String> {
        self.inner.get_intersection_los().map(|l| format!("{l:?}"))
    }

    /// Critical intersection volume-to-capacity ratio X_c (HCM Equation 19-30).
    pub fn get_critical_vc_ratio(&self) -> Option<f64> {
        self.inner.get_critical_vc_ratio()
    }

    /// Approach control delay for "NB", "SB", "EB", or "WB", s/veh
    /// (HCM Equation 19-28). NaN when the approach has no results.
    pub fn approach_delay_s(&self, direction: &str) -> f64 {
        self.inner
            .get_approach_results()
            .iter()
            .find(|a| format!("{:?}", a.direction) == direction.to_uppercase())
            .map(|a| a.control_delay_s)
            .unwrap_or(f64::NAN)
    }

    /// Approach LOS letter for "NB", "SB", "EB", or "WB" (HCM Exhibit 19-8).
    /// Empty string when the approach has no results.
    pub fn approach_los(&self, direction: &str) -> String {
        self.inner
            .get_approach_results()
            .iter()
            .find(|a| format!("{:?}", a.direction) == direction.to_uppercase())
            .map(|a| format!("{:?}", a.los))
            .unwrap_or_default()
    }

    /// Lane-group results (direction, kind, flow rate, saturation flow,
    /// capacity, v/c, delays, LOS, back of queue) as a JS array.
    pub fn lane_groups_to_js_value(&self) -> JsValue {
        serde_wasm_bindgen::to_value(self.inner.get_lane_groups()).unwrap_or(JsValue::NULL)
    }

    pub fn results_to_js_value(&self) -> JsValue {
        let obj = js_sys::Object::new();
        js_sys::Reflect::set(
            &obj,
            &JsValue::from_str("intersection_delay_s"),
            &JsValue::from(self.get_intersection_delay_s().unwrap_or(f64::NAN)),
        )
        .unwrap();
        js_sys::Reflect::set(
            &obj,
            &JsValue::from_str("intersection_los"),
            &JsValue::from_str(&self.get_intersection_los().unwrap_or_default()),
        )
        .unwrap();
        js_sys::Reflect::set(
            &obj,
            &JsValue::from_str("critical_vc_ratio"),
            &JsValue::from(self.get_critical_vc_ratio().unwrap_or(f64::NAN)),
        )
        .unwrap();
        let approaches =
            serde_wasm_bindgen::to_value(self.inner.get_approach_results()).unwrap_or(JsValue::NULL);
        js_sys::Reflect::set(&obj, &JsValue::from_str("approaches"), &approaches).unwrap();

        JsValue::from(obj)
    }
}
