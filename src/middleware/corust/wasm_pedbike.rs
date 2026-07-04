use wasm_bindgen::prelude::*;
use transportations_library::offstreet_pedbike::{
    ExclusivePedestrianFacility, OffStreetBicycleFacility, PedestrianFacilityType,
    PedestrianFlowType, SharedUsePathPedestrian, NUM_PATH_MODES,
};

fn parse_facility_type(s: Option<String>) -> PedestrianFacilityType {
    match s
        .unwrap_or_default()
        .to_lowercase()
        .replace(['-', '_', ' '], "")
        .as_str()
    {
        "crossflow" => PedestrianFacilityType::CrossFlow,
        "stairway" => PedestrianFacilityType::Stairway,
        _ => PedestrianFacilityType::Walkway,
    }
}

fn parse_flow_type(s: Option<String>) -> PedestrianFlowType {
    match s.unwrap_or_default().to_lowercase().as_str() {
        "platooned" | "platoon" => PedestrianFlowType::Platooned,
        _ => PedestrianFlowType::Random,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Exclusive off-street pedestrian facilities (walkways, cross-flow, stairways)
// ═══════════════════════════════════════════════════════════════════════════════

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct WasmExclusivePedestrianFacility {
    inner: ExclusivePedestrianFacility,
}

#[wasm_bindgen]
impl WasmExclusivePedestrianFacility {

    /// Build an exclusive off-street pedestrian facility analysis (HCM Ch.24).
    ///
    /// * `total_walkway_width` — total walkway width W_T, ft.
    /// * `fixed_object_width` — fixed-object effective widths and shy distances W_O, ft.
    /// * `pedestrian_demand` — hourly pedestrian demand v_h, p/h.
    /// * `peak_15min_volume` — field-measured peak 15-min volume, p (used
    ///   directly instead of v_h / (4 × PHF) when provided).
    /// * `phf` — peak hour factor (default 0.85).
    /// * `pedestrian_speed` — average pedestrian speed S_p, ft/min (default 300).
    /// * `facility_type` — "walkway" (default), "cross_flow", or "stairway".
    /// * `flow_type` — "random" (default) or "platooned".
    #[wasm_bindgen(constructor)]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        total_walkway_width: f64,
        fixed_object_width: Option<f64>,
        pedestrian_demand: Option<f64>,
        peak_15min_volume: Option<f64>,
        phf: Option<f64>,
        pedestrian_speed: Option<f64>,
        facility_type: Option<String>,
        flow_type: Option<String>,
    ) -> Self {
        WasmExclusivePedestrianFacility {
            inner: ExclusivePedestrianFacility::new(
                total_walkway_width,
                fixed_object_width.unwrap_or(0.0),
                pedestrian_demand,
                peak_15min_volume,
                phf,
                pedestrian_speed,
                parse_facility_type(facility_type),
                parse_flow_type(flow_type),
            ),
        }
    }

    /// Run the complete methodology (Steps 1-5) and return the LOS letter.
    pub fn analyze(&mut self) -> String {
        let los: char = self.inner.analyze().into();
        los.to_string()
    }

    /// Effective walkway width W_E, ft (HCM Equation 24-1).
    pub fn get_effective_width(&self) -> f64 {
        self.inner.effective_width.unwrap_or(f64::NAN)
    }

    /// Pedestrian volume during the peak 15 min, p (HCM Equation 24-2).
    pub fn get_flow_rate_15min(&self) -> f64 {
        self.inner.flow_rate_15min.unwrap_or(f64::NAN)
    }

    /// Pedestrian flow per unit width v_p, p/ft/min (HCM Equation 24-3).
    pub fn get_unit_flow_rate(&self) -> f64 {
        self.inner.unit_flow_rate.unwrap_or(f64::NAN)
    }

    /// Average pedestrian space A_p, ft²/p (HCM Equation 24-4). Infinity for
    /// an empty facility.
    pub fn get_pedestrian_space(&self) -> f64 {
        self.inner.pedestrian_space.unwrap_or(f64::INFINITY)
    }

    /// Volume-to-capacity ratio.
    pub fn get_vc_ratio(&self) -> f64 {
        self.inner.vc_ratio.unwrap_or(f64::NAN)
    }

    pub fn results_to_js_value(&self) -> JsValue {
        let obj = js_sys::Object::new();
        js_sys::Reflect::set(&obj, &JsValue::from_str("effective_width"), &JsValue::from(self.get_effective_width())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("flow_rate_15min"), &JsValue::from(self.get_flow_rate_15min())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("unit_flow_rate"), &JsValue::from(self.get_unit_flow_rate())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("pedestrian_space"), &JsValue::from(self.get_pedestrian_space())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("vc_ratio"), &JsValue::from(self.get_vc_ratio())).unwrap();

        JsValue::from(obj)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Pedestrians on shared-use paths
// ═══════════════════════════════════════════════════════════════════════════════

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct WasmSharedUsePathPedestrian {
    inner: SharedUsePathPedestrian,
}

#[wasm_bindgen]
impl WasmSharedUsePathPedestrian {

    /// Build a shared-use path pedestrian LOS analysis (HCM Ch.24).
    ///
    /// * `bicycle_demand_same_direction` — Q_sb, bicycles/h.
    /// * `bicycle_demand_opposing` — Q_ob, bicycles/h.
    /// * `phf` — peak hour factor (default 0.85).
    /// * `pedestrian_speed` — mean pedestrian speed S_p (default 3.4 mi/h;
    ///   only the ratio S_p / S_b is used).
    /// * `bicycle_speed` — mean bicycle speed S_b (default 12.8 mi/h).
    /// * `is_one_way` — one-way path flag (no meeting events).
    #[wasm_bindgen(constructor)]
    pub fn new(
        bicycle_demand_same_direction: Option<f64>,
        bicycle_demand_opposing: Option<f64>,
        phf: Option<f64>,
        pedestrian_speed: Option<f64>,
        bicycle_speed: Option<f64>,
        is_one_way: Option<bool>,
    ) -> Self {
        let mut inner = SharedUsePathPedestrian::new(
            bicycle_demand_same_direction,
            bicycle_demand_opposing,
            phf,
            pedestrian_speed,
            bicycle_speed,
        );
        inner.is_one_way = is_one_way.unwrap_or(false);
        WasmSharedUsePathPedestrian { inner }
    }

    /// Run the complete methodology (Steps 1-3) and return the LOS letter.
    pub fn analyze(&mut self) -> String {
        let los: char = self.inner.analyze().into();
        los.to_string()
    }

    /// Number of passing events F_p, events/h (HCM Equation 24-5).
    pub fn get_passing_events(&self) -> f64 {
        self.inner.passing_events.unwrap_or(f64::NAN)
    }

    /// Number of meeting events F_m, events/h (HCM Equation 24-6).
    pub fn get_meeting_events(&self) -> f64 {
        self.inner.meeting_events.unwrap_or(f64::NAN)
    }

    /// Total weighted events F = F_p + 0.5 F_m, events/h (HCM Equation 24-7).
    pub fn get_total_events(&self) -> f64 {
        self.inner.total_events.unwrap_or(f64::NAN)
    }

    pub fn results_to_js_value(&self) -> JsValue {
        let obj = js_sys::Object::new();
        js_sys::Reflect::set(&obj, &JsValue::from_str("passing_events"), &JsValue::from(self.get_passing_events())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("meeting_events"), &JsValue::from(self.get_meeting_events())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("total_events"), &JsValue::from(self.get_total_events())).unwrap();

        JsValue::from(obj)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Off-street bicycle facilities (BLOS)
// ═══════════════════════════════════════════════════════════════════════════════

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct WasmOffStreetBicycleFacility {
    inner: OffStreetBicycleFacility,
}

#[wasm_bindgen]
impl WasmOffStreetBicycleFacility {

    /// Build a bicycle LOS (BLOS) analysis for a shared-use or exclusive
    /// off-street bicycle facility (HCM Ch.24).
    ///
    /// * `path_width` — path width, ft (methodology applies up to 20 ft).
    /// * `segment_length` — path segment length L, mi.
    /// * `has_centerline` — centerline stripe present.
    /// * `two_way_demand` — total two-directional path demand, users/h.
    /// * `directional_split` — subject-direction share of demand (default 0.50).
    /// * `phf` — peak hour factor (default 0.85).
    /// * `is_one_way` — one-way path flag (no opposing users).
    /// * `exclusive_bicycle` — true for an exclusive bicycle facility (the
    ///   bicycle mode split is set to 1.0 and all other modes to zero).
    /// * `mode_splits`, `mode_speeds`, `mode_speed_sds` — optional 5-value
    ///   overrides for [bicycle, pedestrian, runner, inline skater, child
    ///   bicyclist] (defaults from HCM Exhibit 24-6).
    #[wasm_bindgen(constructor)]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        path_width: f64,
        segment_length: f64,
        has_centerline: Option<bool>,
        two_way_demand: Option<f64>,
        directional_split: Option<f64>,
        phf: Option<f64>,
        is_one_way: Option<bool>,
        exclusive_bicycle: Option<bool>,
        mode_splits: Option<Vec<f64>>,
        mode_speeds: Option<Vec<f64>>,
        mode_speed_sds: Option<Vec<f64>>,
    ) -> Self {
        let mut inner = OffStreetBicycleFacility::new(
            path_width,
            segment_length,
            has_centerline.unwrap_or(false),
            two_way_demand,
            directional_split,
            phf,
        );
        inner.is_one_way = is_one_way.unwrap_or(false);
        if exclusive_bicycle.unwrap_or(false) {
            for (i, group) in inner.user_groups.iter_mut().enumerate() {
                group.mode_split = if i == 0 { 1.0 } else { 0.0 };
            }
        }
        if let Some(splits) = mode_splits {
            for (group, v) in inner.user_groups.iter_mut().zip(splits.into_iter().take(NUM_PATH_MODES)) {
                group.mode_split = v;
            }
        }
        if let Some(speeds) = mode_speeds {
            for (group, v) in inner.user_groups.iter_mut().zip(speeds.into_iter().take(NUM_PATH_MODES)) {
                group.average_speed = v;
            }
        }
        if let Some(sds) = mode_speed_sds {
            for (group, v) in inner.user_groups.iter_mut().zip(sds.into_iter().take(NUM_PATH_MODES)) {
                group.speed_standard_deviation = v;
            }
        }
        WasmOffStreetBicycleFacility { inner }
    }

    /// Run the complete BLOS methodology (Steps 1-8, including the low-volume
    /// adjustment) and return the LOS letter.
    pub fn analyze(&mut self) -> String {
        let los: char = self.inner.analyze().into();
        los.to_string()
    }

    /// Active passings per minute A_T (HCM Equation 24-12).
    pub fn get_active_passings_per_minute(&self) -> f64 {
        self.inner.active_passings_per_minute.unwrap_or(f64::NAN)
    }

    /// Meetings per minute M_T (HCM Equation 24-16).
    pub fn get_meetings_per_minute(&self) -> f64 {
        self.inner.meetings_per_minute.unwrap_or(f64::NAN)
    }

    /// Number of effective lanes (HCM Exhibit 24-14).
    pub fn get_effective_lanes(&self) -> u32 {
        u32::from(self.inner.effective_lanes.unwrap_or(0))
    }

    /// Total probability of delayed passing P_Tds (HCM Equation 24-33).
    pub fn get_probability_delayed_passing(&self) -> f64 {
        self.inner.total_probability_delayed_passing.unwrap_or(f64::NAN)
    }

    /// Delayed passings per minute DP_m (HCM Equation 24-34).
    pub fn get_delayed_passings_per_minute(&self) -> f64 {
        self.inner.delayed_passings_per_minute.unwrap_or(f64::NAN)
    }

    /// Weighted events per minute E = M_T + 10 A_T (HCM Equation 24-35 term).
    pub fn get_weighted_events_per_minute(&self) -> f64 {
        self.inner.weighted_events_per_minute.unwrap_or(f64::NAN)
    }

    /// BLOS score (HCM Equation 24-35).
    pub fn get_blos_score(&self) -> f64 {
        self.inner.blos_score.unwrap_or(f64::NAN)
    }

    pub fn results_to_js_value(&self) -> JsValue {
        let obj = js_sys::Object::new();
        js_sys::Reflect::set(&obj, &JsValue::from_str("active_passings_per_minute"), &JsValue::from(self.get_active_passings_per_minute())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("meetings_per_minute"), &JsValue::from(self.get_meetings_per_minute())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("effective_lanes"), &JsValue::from(self.get_effective_lanes())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("probability_delayed_passing"), &JsValue::from(self.get_probability_delayed_passing())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("delayed_passings_per_minute"), &JsValue::from(self.get_delayed_passings_per_minute())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("weighted_events_per_minute"), &JsValue::from(self.get_weighted_events_per_minute())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("blos_score"), &JsValue::from(self.get_blos_score())).unwrap();

        JsValue::from(obj)
    }
}
