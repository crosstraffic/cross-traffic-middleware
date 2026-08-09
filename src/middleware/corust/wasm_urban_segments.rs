use wasm_bindgen::prelude::*;
use transportations_library::urban_segments::{
    AccessPointApproach, BoundaryControlType, UrbanSegment,
};

/// Parse the UI control-type string into the Chapter 18 boundary control enum.
pub(crate) fn parse_boundary_control(control: &str) -> BoundaryControlType {
    match control.to_lowercase().as_str() {
        "signalized" | "signal" => BoundaryControlType::Signalized,
        "allwaystop" | "all_way_stop" | "awsc" | "stop" => BoundaryControlType::AllWayStop,
        "yield" | "yieldcontrolled" | "yield_controlled" => BoundaryControlType::YieldControlled,
        "roundabout" => BoundaryControlType::Roundabout,
        _ => BoundaryControlType::Uncontrolled,
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct WasmUrbanSegment {
    inner: UrbanSegment,
}

#[wasm_bindgen]
impl WasmUrbanSegment {

    /// Build a Chapter 18 segment. Everything after `control` is optional
    /// and takes the Exhibit 18-5 default when omitted.
    ///
    /// The last nine arguments drive Step 2's access-point delay term
    /// `Σ d_ap,i` of Equation 18-7. The library picks among three sources in
    /// order: the Chapter 30, Section 4 procedure when access-point
    /// approaches have been added with [`Self::add_access_point`]; then
    /// `access_point_delays_s`, the per-point delays supplied directly (the
    /// Exhibit 30-35 published values take this path); then the Exhibit
    /// 18-13 planning estimate, whose inputs are
    /// `n_influential_access_points` (default `N_ap,s + p_ap,lt N_ap,o`),
    /// the two turn percentages (Exhibit 18-13 assumes 10% each), and the
    /// two turn-bay adequacy flags.
    #[wasm_bindgen(constructor)]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        segment_length_ft: f64,
        n_through_lanes: u32,
        speed_limit_mph: f64,
        through_demand_veh_h: f64,
        control: String,
        upstream_intersection_width_ft: Option<f64>,
        restrictive_median_length_ft: Option<f64>,
        proportion_with_curb: Option<f64>,
        proportion_on_street_parking: Option<f64>,
        n_access_points_subject: Option<f64>,
        n_access_points_opposing: Option<f64>,
        prop_opposing_left_accessible: Option<f64>,
        signal_spacing_ft: Option<f64>,
        free_flow_speed_override_mph: Option<f64>,
        midsegment_flow_veh_h: Option<f64>,
        through_capacity_veh_h: Option<f64>,
        through_control_delay_s: Option<f64>,
        cycle_length_s: Option<f64>,
        effective_green_s: Option<f64>,
        arrival_type: Option<u32>,
        platoon_ratio: Option<f64>,
        sat_flow_veh_h_ln: Option<f64>,
        stopped_vehicles_veh_ln: Option<f64>,
        queue2_veh_ln: Option<f64>,
        queue3_veh_ln: Option<f64>,
        full_stop_rate_override: Option<f64>,
        stop_rate_other: Option<f64>,
        prop_left_turn_lanes: Option<f64>,
        access_point_delays_s: Option<Vec<f64>>,
        n_influential_access_points: Option<f64>,
        pct_left_turns_access: Option<f64>,
        pct_right_turns_access: Option<f64>,
        access_left_bay_adequate: Option<bool>,
        access_right_bay_adequate: Option<bool>,
        midsegment_other_delay_s: Option<f64>,
        analysis_period_h: Option<f64>,
        access_point_turn_delay_speed_mph: Option<f64>,
    ) -> Self {
        let mut inner = UrbanSegment::new(
            segment_length_ft,
            n_through_lanes,
            speed_limit_mph,
            through_demand_veh_h,
            parse_boundary_control(&control),
        );
        if let Some(v) = upstream_intersection_width_ft {
            inner.upstream_intersection_width_ft = v;
        }
        if let Some(v) = restrictive_median_length_ft {
            inner.restrictive_median_length_ft = v;
        }
        if let Some(v) = proportion_with_curb {
            inner.proportion_with_curb = v;
        }
        if let Some(v) = proportion_on_street_parking {
            inner.proportion_on_street_parking = v;
        }
        if let Some(v) = n_access_points_subject {
            inner.n_access_points_subject = v;
        }
        if let Some(v) = n_access_points_opposing {
            inner.n_access_points_opposing = v;
        }
        if let Some(v) = prop_opposing_left_accessible {
            inner.prop_opposing_left_accessible = v;
        }
        if signal_spacing_ft.is_some() {
            inner.signal_spacing_ft = signal_spacing_ft;
        }
        if free_flow_speed_override_mph.is_some() {
            inner.free_flow_speed_override_mph = free_flow_speed_override_mph;
        }
        if midsegment_flow_veh_h.is_some() {
            inner.midsegment_flow_veh_h = midsegment_flow_veh_h;
        }
        if through_capacity_veh_h.is_some() {
            inner.through_capacity_veh_h = through_capacity_veh_h;
        }
        if through_control_delay_s.is_some() {
            inner.through_control_delay_s = through_control_delay_s;
        }
        if cycle_length_s.is_some() {
            inner.cycle_length_s = cycle_length_s;
        }
        if effective_green_s.is_some() {
            inner.effective_green_s = effective_green_s;
        }
        if let Some(v) = arrival_type {
            inner.arrival_type = Some(v as u8);
        }
        if platoon_ratio.is_some() {
            inner.platoon_ratio = platoon_ratio;
        }
        if sat_flow_veh_h_ln.is_some() {
            inner.sat_flow_veh_h_ln = sat_flow_veh_h_ln;
        }
        if stopped_vehicles_veh_ln.is_some() {
            inner.stopped_vehicles_veh_ln = stopped_vehicles_veh_ln;
        }
        if queue2_veh_ln.is_some() {
            inner.queue2_veh_ln = queue2_veh_ln;
        }
        if queue3_veh_ln.is_some() {
            inner.queue3_veh_ln = queue3_veh_ln;
        }
        if full_stop_rate_override.is_some() {
            inner.full_stop_rate_override = full_stop_rate_override;
        }
        if let Some(v) = stop_rate_other {
            inner.stop_rate_other = v;
        }
        if prop_left_turn_lanes.is_some() {
            inner.prop_left_turn_lanes = prop_left_turn_lanes;
        }
        if access_point_delays_s.is_some() {
            inner.access_point_delays_s = access_point_delays_s;
        }
        if n_influential_access_points.is_some() {
            inner.n_influential_access_points = n_influential_access_points;
        }
        if let Some(v) = pct_left_turns_access {
            inner.pct_left_turns_access = v;
        }
        if let Some(v) = pct_right_turns_access {
            inner.pct_right_turns_access = v;
        }
        if let Some(v) = access_left_bay_adequate {
            inner.access_left_bay_adequate = v;
        }
        if let Some(v) = access_right_bay_adequate {
            inner.access_right_bay_adequate = v;
        }
        if let Some(v) = midsegment_other_delay_s {
            inner.midsegment_other_delay_s = v;
        }
        if let Some(v) = analysis_period_h {
            inner.analysis_period_h = v;
        }
        if access_point_turn_delay_speed_mph.is_some() {
            inner.access_point_turn_delay_speed_mph = access_point_turn_delay_speed_mph;
        }
        WasmUrbanSegment { inner }
    }

    /// Append one active access point approach in the subject direction of
    /// travel, switching Step 2 to the computed Chapter 30, Section 4
    /// procedure (Equations 30-31 through 30-68) for every access point on
    /// the segment. Call once per point, before `analyze()`.
    ///
    /// `approach` is an object matching the serde schema of
    /// `hcm::urban_segments::access_point_delay::AccessPointApproach`:
    ///
    /// ```json
    /// { "v_lt": 74.0, "v_th": 1002.0, "v_rt": 74.0,
    ///   "n_sl": 1, "n_t": 0, "n_sr": 1,
    ///   "opposing_flow_veh_h": 1076.0,
    ///   "left_turn_bay": false, "right_turn_bay": false,
    ///   "n_lt_lanes": 0, "left_bay_storage_ft": 0.0,
    ///   "pct_heavy_veh": 0.0 }
    /// ```
    ///
    /// The right-turn branch is evaluated at the posted speed limit unless
    /// `access_point_turn_delay_speed_mph` overrides it; that is what
    /// reproduces the Exhibit 30-35 published per-point delays (0.193 and
    /// 0.194 s/veh) and the 0.115 inside-lane blockage probability. See the
    /// VERIFY-HCM note in the library's `access_point_delay` module docs.
    pub fn add_access_point(&mut self, approach: JsValue) -> Result<(), JsValue> {
        let approach: AccessPointApproach = serde_wasm_bindgen::from_value(approach)
            .map_err(|e| JsValue::from_str(&format!("invalid access point approach: {e}")))?;
        self.inner
            .access_point_approaches
            .get_or_insert_with(Vec::new)
            .push(approach);
        Ok(())
    }

    /// Run the full HCM Ch.18 motorized vehicle pipeline (Steps 1-3 and
    /// 5-10) and return the segment LOS letter (Exhibit 18-1).
    /// Populates free-flow speed, running time, travel speed, stop rates,
    /// v/c ratio, and the perception score.
    pub fn analyze(&mut self) -> String {
        self.inner.analyze();
        self.get_los()
    }

    /// Speed constant S_0, mi/h (Exhibit 18-11, note a).
    pub fn get_speed_constant(&self) -> Option<f64> {
        self.inner.speed_constant_mph
    }

    /// Cross-section adjustment f_CS, mi/h (Exhibit 18-11, note b).
    pub fn get_f_cs(&self) -> Option<f64> {
        self.inner.f_cs_mph
    }

    /// Access point adjustment f_A, mi/h (Exhibit 18-11, note c).
    pub fn get_f_a(&self) -> Option<f64> {
        self.inner.f_a_mph
    }

    /// On-street parking adjustment f_pk, mi/h (Exhibit 18-11, note d).
    pub fn get_f_pk(&self) -> Option<f64> {
        self.inner.f_pk_mph
    }

    /// Signal spacing adjustment factor f_L (Equation 18-4).
    pub fn get_f_l(&self) -> Option<f64> {
        self.inner.f_l
    }

    /// Vehicle proximity adjustment factor f_v (Equation 18-6).
    pub fn get_f_v(&self) -> Option<f64> {
        self.inner.f_v
    }

    pub fn get_base_ffs(&self) -> Option<f64> {
        self.inner.base_ffs_mph
    }

    pub fn get_free_flow_speed(&self) -> Option<f64> {
        self.inner.free_flow_speed_mph
    }

    pub fn get_running_time(&self) -> Option<f64> {
        self.inner.running_time_s
    }

    pub fn get_running_speed(&self) -> Option<f64> {
        self.inner.running_speed_mph
    }

    pub fn get_proportion_arriving_green(&self) -> Option<f64> {
        self.inner.proportion_arriving_green
    }

    pub fn get_access_point_delay(&self) -> Option<f64> {
        self.inner.access_point_delay_total_s
    }

    /// Per-access-point Chapter 30, Section 4 breakdown as a JS array of
    /// `{ delay_left_s, delay_right_s, delay_total_s,
    /// prob_inside_lane_blocked }` in the order the approaches were added
    /// (the Exhibit 30-35 columns). `null` unless approaches were supplied
    /// through [`Self::add_access_point`] and `analyze()` has run.
    pub fn access_point_delays_computed(&self) -> JsValue {
        match &self.inner.access_point_delays_computed {
            Some(delays) => serde_wasm_bindgen::to_value(delays).unwrap_or(JsValue::NULL),
            None => JsValue::NULL,
        }
    }

    pub fn get_through_delay(&self) -> Option<f64> {
        self.inner.through_delay_s
    }

    pub fn get_full_stop_rate(&self) -> Option<f64> {
        self.inner.full_stop_rate
    }

    pub fn get_travel_speed(&self) -> Option<f64> {
        self.inner.travel_speed_mph
    }

    pub fn get_spatial_stop_rate(&self) -> Option<f64> {
        self.inner.spatial_stop_rate_stops_mi
    }

    pub fn get_vc_ratio(&self) -> Option<f64> {
        self.inner.vc_ratio
    }

    pub fn get_demand_exceeds_capacity(&self) -> Option<bool> {
        self.inner.demand_exceeds_capacity
    }

    pub fn get_perception_score(&self) -> Option<f64> {
        self.inner.perception_score
    }

    pub fn get_los(&self) -> String {
        self.inner
            .los
            .map(|l| format!("{l:?}"))
            .unwrap_or_default()
    }

    pub fn results_to_js_value(&self) -> JsValue {
        let opt = |v: Option<f64>| v.map(JsValue::from).unwrap_or(JsValue::NULL);
        let obj = js_sys::Object::new();
        js_sys::Reflect::set(&obj, &JsValue::from_str("speed_constant"), &opt(self.inner.speed_constant_mph)).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("f_cs"), &opt(self.inner.f_cs_mph)).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("f_a"), &opt(self.inner.f_a_mph)).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("f_pk"), &opt(self.inner.f_pk_mph)).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("base_ffs"), &opt(self.inner.base_ffs_mph)).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("f_l"), &opt(self.inner.f_l)).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("free_flow_speed"), &opt(self.inner.free_flow_speed_mph)).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("f_v"), &opt(self.inner.f_v)).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("running_time"), &opt(self.inner.running_time_s)).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("running_speed"), &opt(self.inner.running_speed_mph)).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("proportion_arriving_green"), &opt(self.inner.proportion_arriving_green)).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("access_point_delay"), &opt(self.inner.access_point_delay_total_s)).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("access_point_delays_computed"), &self.access_point_delays_computed()).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("through_delay"), &opt(self.inner.through_delay_s)).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("full_stop_rate"), &opt(self.inner.full_stop_rate)).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("travel_speed"), &opt(self.inner.travel_speed_mph)).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("spatial_stop_rate"), &opt(self.inner.spatial_stop_rate_stops_mi)).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("vc_ratio"), &opt(self.inner.vc_ratio)).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("perception_score"), &opt(self.inner.perception_score)).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("los"), &JsValue::from_str(&self.get_los())).unwrap();

        JsValue::from(obj)
    }
}
