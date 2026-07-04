use wasm_bindgen::prelude::*;
use transportations_library::urban_facilities::UrbanFacility;
use transportations_library::urban_segments::UrbanSegment;

use super::wasm_urban_segments::parse_boundary_control;

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct WasmUrbanFacility {
    inner: UrbanFacility,
}

#[wasm_bindgen]
impl WasmUrbanFacility {

    #[wasm_bindgen(constructor)]
    pub fn new(prop_left_turn_lanes: Option<f64>) -> Self {
        let mut inner = UrbanFacility::new(Vec::new());
        if prop_left_turn_lanes.is_some() {
            inner.prop_left_turn_lanes = prop_left_turn_lanes;
        }
        WasmUrbanFacility { inner }
    }

    /// Append a Chapter 18 segment (ordered upstream to downstream) to the
    /// facility in the subject direction of travel.
    #[allow(clippy::too_many_arguments)]
    pub fn add_segment(
        &mut self,
        segment_length_ft: f64,
        n_through_lanes: u32,
        speed_limit_mph: f64,
        through_demand_veh_h: f64,
        control: String,
        n_access_points_subject: Option<f64>,
        n_access_points_opposing: Option<f64>,
        midsegment_flow_veh_h: Option<f64>,
        through_capacity_veh_h: Option<f64>,
        through_control_delay_s: Option<f64>,
        cycle_length_s: Option<f64>,
        effective_green_s: Option<f64>,
        platoon_ratio: Option<f64>,
        sat_flow_veh_h_ln: Option<f64>,
        full_stop_rate_override: Option<f64>,
    ) {
        let mut segment = UrbanSegment::new(
            segment_length_ft,
            n_through_lanes,
            speed_limit_mph,
            through_demand_veh_h,
            parse_boundary_control(&control),
        );
        if let Some(v) = n_access_points_subject {
            segment.n_access_points_subject = v;
        }
        if let Some(v) = n_access_points_opposing {
            segment.n_access_points_opposing = v;
        }
        if midsegment_flow_veh_h.is_some() {
            segment.midsegment_flow_veh_h = midsegment_flow_veh_h;
        }
        if through_capacity_veh_h.is_some() {
            segment.through_capacity_veh_h = through_capacity_veh_h;
        }
        if through_control_delay_s.is_some() {
            segment.through_control_delay_s = through_control_delay_s;
        }
        if cycle_length_s.is_some() {
            segment.cycle_length_s = cycle_length_s;
        }
        if effective_green_s.is_some() {
            segment.effective_green_s = effective_green_s;
        }
        if platoon_ratio.is_some() {
            segment.platoon_ratio = platoon_ratio;
        }
        if sat_flow_veh_h_ln.is_some() {
            segment.sat_flow_veh_h_ln = sat_flow_veh_h_ln;
        }
        if full_stop_rate_override.is_some() {
            segment.full_stop_rate_override = full_stop_rate_override;
        }
        self.inner.segments.push(segment);
    }

    /// Run the full HCM Ch.16 pipeline: evaluate every segment with the
    /// Chapter 18 engine, then aggregate (Equations 16-2 through 16-4 and
    /// the Exhibit 16-3 LOS). Returns the facility LOS letter.
    pub fn analyze(&mut self) -> Result<String, JsValue> {
        self.inner
            .analyze()
            .map(|r| format!("{:?}", r.los))
            .map_err(|e| JsValue::from_str(&e))
    }

    pub fn num_segments(&self) -> u32 {
        self.inner.num_segments() as u32
    }

    pub fn get_length_ft(&self) -> f64 {
        self.inner.length_ft()
    }

    pub fn get_base_ffs(&self) -> Option<f64> {
        self.inner.get_base_ffs_mph()
    }

    pub fn get_travel_speed(&self) -> Option<f64> {
        self.inner.get_travel_speed_mph()
    }

    pub fn get_travel_time(&self) -> Option<f64> {
        self.inner.results.as_ref().map(|r| r.travel_time_s)
    }

    pub fn get_base_free_flow_travel_time(&self) -> Option<f64> {
        self.inner.results.as_ref().map(|r| r.base_free_flow_travel_time_s)
    }

    pub fn get_spatial_stop_rate(&self) -> Option<f64> {
        self.inner.get_spatial_stop_rate()
    }

    pub fn get_critical_vc_ratio(&self) -> Option<f64> {
        self.inner.get_critical_vc_ratio()
    }

    pub fn get_perception_score(&self) -> Option<f64> {
        self.inner.get_perception_score()
    }

    pub fn get_los(&self) -> String {
        self.inner
            .get_los()
            .map(|l| format!("{l:?}"))
            .unwrap_or_default()
    }

    pub fn get_poorest_segment_los(&self) -> String {
        self.inner
            .get_poorest_segment_los()
            .map(|l| format!("{l:?}"))
            .unwrap_or_default()
    }

    /// Per-segment results (travel speed, base FFS, spatial stop rate, v/c
    /// ratio, LOS) as an array of plain objects, ordered upstream to
    /// downstream.
    pub fn segments_to_js_value(&self) -> JsValue {
        let opt = |v: Option<f64>| v.map(JsValue::from).unwrap_or(JsValue::NULL);
        let js_array = js_sys::Array::new();
        for seg in self.inner.segments.iter() {
            let obj = js_sys::Object::new();
            js_sys::Reflect::set(&obj, &JsValue::from_str("length_ft"), &JsValue::from(seg.segment_length_ft)).unwrap();
            js_sys::Reflect::set(&obj, &JsValue::from_str("base_ffs"), &opt(seg.base_ffs_mph)).unwrap();
            js_sys::Reflect::set(&obj, &JsValue::from_str("travel_speed"), &opt(seg.travel_speed_mph)).unwrap();
            js_sys::Reflect::set(&obj, &JsValue::from_str("spatial_stop_rate"), &opt(seg.spatial_stop_rate_stops_mi)).unwrap();
            js_sys::Reflect::set(&obj, &JsValue::from_str("vc_ratio"), &opt(seg.vc_ratio)).unwrap();
            let los = seg.los.map(|l| format!("{l:?}")).unwrap_or_default();
            js_sys::Reflect::set(&obj, &JsValue::from_str("los"), &JsValue::from_str(&los)).unwrap();
            js_array.push(&obj);
        }
        JsValue::from(js_array)
    }

    pub fn results_to_js_value(&self) -> JsValue {
        let opt = |v: Option<f64>| v.map(JsValue::from).unwrap_or(JsValue::NULL);
        let obj = js_sys::Object::new();
        js_sys::Reflect::set(&obj, &JsValue::from_str("length_ft"), &JsValue::from(self.get_length_ft())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("base_ffs"), &opt(self.get_base_ffs())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("travel_speed"), &opt(self.get_travel_speed())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("travel_time"), &opt(self.get_travel_time())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("base_free_flow_travel_time"), &opt(self.get_base_free_flow_travel_time())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("spatial_stop_rate"), &opt(self.get_spatial_stop_rate())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("critical_vc_ratio"), &opt(self.get_critical_vc_ratio())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("perception_score"), &opt(self.get_perception_score())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("los"), &JsValue::from_str(&self.get_los())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("poorest_segment_los"), &JsValue::from_str(&self.get_poorest_segment_los())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("segments"), &self.segments_to_js_value()).unwrap();

        JsValue::from(obj)
    }
}
