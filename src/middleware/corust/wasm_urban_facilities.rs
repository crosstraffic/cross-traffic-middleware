use wasm_bindgen::prelude::*;
use transportations_library::hcm::common::LevelOfService;
use transportations_library::urban_facilities::UrbanFacility;
use transportations_library::urban_segments::{BoundaryControlType, UrbanSegment};

use super::wasm_urban_segments::parse_boundary_control;

/// Parse a LOS letter into the library enum. Anything else is rejected
/// rather than defaulted, because a silently wrong letter would move the
/// poorest-performing-segment report without changing any other output.
fn parse_los(letter: &str) -> Result<LevelOfService, JsValue> {
    match letter.trim().to_ascii_uppercase().as_str() {
        "A" => Ok(LevelOfService::A),
        "B" => Ok(LevelOfService::B),
        "C" => Ok(LevelOfService::C),
        "D" => Ok(LevelOfService::D),
        "E" => Ok(LevelOfService::E),
        "F" => Ok(LevelOfService::F),
        other => Err(JsValue::from_str(&format!(
            "unknown LOS letter {other:?}: expected A through F"
        ))),
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct WasmUrbanFacility {
    inner: UrbanFacility,
    /// Number of segments added through [`WasmUrbanFacility::add_segment_summary`].
    /// Those carry published measures instead of Chapter 18 inputs, so
    /// `analyze()` must refuse to run the engine over them.
    n_summary_segments: usize,
}

#[wasm_bindgen]
impl WasmUrbanFacility {

    #[wasm_bindgen(constructor)]
    pub fn new(prop_left_turn_lanes: Option<f64>) -> Self {
        let mut inner = UrbanFacility::new(Vec::new());
        if prop_left_turn_lanes.is_some() {
            inner.prop_left_turn_lanes = prop_left_turn_lanes;
        }
        WasmUrbanFacility { inner, n_summary_segments: 0 }
    }

    /// Append a Chapter 18 segment (ordered upstream to downstream) to the
    /// facility in the subject direction of travel. Everything after
    /// `control` is optional; the trailing arguments mirror
    /// `WasmUrbanSegment`'s constructor, including the nine that select
    /// among the three Equation 18-7 access-point delay sources.
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
        upstream_intersection_width_ft: Option<f64>,
        restrictive_median_length_ft: Option<f64>,
        proportion_with_curb: Option<f64>,
        proportion_on_street_parking: Option<f64>,
        prop_opposing_left_accessible: Option<f64>,
        signal_spacing_ft: Option<f64>,
        free_flow_speed_override_mph: Option<f64>,
        access_point_delays_s: Option<Vec<f64>>,
        n_influential_access_points: Option<f64>,
        pct_left_turns_access: Option<f64>,
        pct_right_turns_access: Option<f64>,
        access_left_bay_adequate: Option<bool>,
        access_right_bay_adequate: Option<bool>,
        midsegment_other_delay_s: Option<f64>,
        analysis_period_h: Option<f64>,
        access_point_turn_delay_speed_mph: Option<f64>,
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
        if let Some(v) = upstream_intersection_width_ft {
            segment.upstream_intersection_width_ft = v;
        }
        if let Some(v) = restrictive_median_length_ft {
            segment.restrictive_median_length_ft = v;
        }
        if let Some(v) = proportion_with_curb {
            segment.proportion_with_curb = v;
        }
        if let Some(v) = proportion_on_street_parking {
            segment.proportion_on_street_parking = v;
        }
        if let Some(v) = prop_opposing_left_accessible {
            segment.prop_opposing_left_accessible = v;
        }
        if signal_spacing_ft.is_some() {
            segment.signal_spacing_ft = signal_spacing_ft;
        }
        if free_flow_speed_override_mph.is_some() {
            segment.free_flow_speed_override_mph = free_flow_speed_override_mph;
        }
        if access_point_delays_s.is_some() {
            segment.access_point_delays_s = access_point_delays_s;
        }
        if n_influential_access_points.is_some() {
            segment.n_influential_access_points = n_influential_access_points;
        }
        if let Some(v) = pct_left_turns_access {
            segment.pct_left_turns_access = v;
        }
        if let Some(v) = pct_right_turns_access {
            segment.pct_right_turns_access = v;
        }
        if let Some(v) = access_left_bay_adequate {
            segment.access_left_bay_adequate = v;
        }
        if let Some(v) = access_right_bay_adequate {
            segment.access_right_bay_adequate = v;
        }
        if let Some(v) = midsegment_other_delay_s {
            segment.midsegment_other_delay_s = v;
        }
        if let Some(v) = analysis_period_h {
            segment.analysis_period_h = v;
        }
        if access_point_turn_delay_speed_mph.is_some() {
            segment.access_point_turn_delay_speed_mph = access_point_turn_delay_speed_mph;
        }
        self.inner.segments.push(segment);
    }

    /// Append a segment from a configuration object matching the serde
    /// schema of the library's `UrbanSegment` — the same shape the library's
    /// own fixture files use (e.g. the `segments` entries of
    /// `tests/ExampleCases/hcm/UrbanFacilities/case3.json`), so a fixture
    /// segment is loadable verbatim in one call instead of through the
    /// 31-argument positional [`Self::add_segment`]. Any omitted field keeps
    /// the library default; unknown fields are ignored, so misspelling a
    /// field name silently falls back to that default — prefer copying field
    /// names from the fixture files.
    pub fn add_segment_from_config(&mut self, config: JsValue) -> Result<(), JsValue> {
        let segment: UrbanSegment = serde_wasm_bindgen::from_value(config)
            .map_err(|e| JsValue::from_str(&format!("invalid urban segment configuration: {e}")))?;
        self.inner.segments.push(segment);
        Ok(())
    }

    /// Append a segment described by its already-known Chapter 18
    /// performance measures rather than by its inputs — the Exhibit 16-7
    /// "HCM method output" case, and the one the published example problems
    /// take (Chapter 29 Example Problem 1 publishes per-segment base FFS,
    /// travel speed, and spatial stop rate, not the geometry behind them).
    ///
    /// The arguments are the fields of the library's `SegmentSummary`. Use
    /// with [`Self::aggregate`], which runs Chapter 16 Steps 1-4 over these
    /// measures; `analyze()` refuses to run once any summary segment is
    /// present, because there are no Chapter 18 inputs behind it to
    /// recompute from.
    ///
    /// * `spatial_stop_rate_stops_mi` — omit on any segment and the
    ///   Equation 16-4 facility stop rate (and the perception score built on
    ///   it) is reported as `undefined` rather than aggregated from a
    ///   partial set.
    /// * `vc_ratio` — the through movement's ratio at the segment's
    ///   downstream boundary intersection; the largest becomes the critical
    ///   ratio of the Exhibit 16-3 footnote.
    /// * `los` — the segment LOS letter, for the poorest-performing-segment
    ///   report of Step 4.
    pub fn add_segment_summary(
        &mut self,
        length_ft: f64,
        base_ffs_mph: f64,
        travel_speed_mph: f64,
        spatial_stop_rate_stops_mi: Option<f64>,
        vc_ratio: Option<f64>,
        los: Option<String>,
    ) -> Result<(), JsValue> {
        let los = los.map(|l| parse_los(&l)).transpose()?;
        // The Chapter 18 inputs are placeholders: Steps 1-4 read only the
        // length and the computed measures set below.
        let mut segment = UrbanSegment::new(
            length_ft,
            1,
            0.0,
            0.0,
            BoundaryControlType::Signalized,
        );
        segment.base_ffs_mph = Some(base_ffs_mph);
        segment.travel_speed_mph = Some(travel_speed_mph);
        segment.spatial_stop_rate_stops_mi = spatial_stop_rate_stops_mi;
        segment.vc_ratio = vc_ratio;
        segment.los = los;
        self.inner.segments.push(segment);
        self.n_summary_segments += 1;
        Ok(())
    }

    /// Run the Chapter 16 aggregation (Equations 16-2 through 16-4 and the
    /// Exhibit 16-3 LOS) over the per-segment measures already held, without
    /// re-running the Chapter 18 engine. Returns the facility LOS letter.
    /// Use after [`Self::add_segment_summary`], or after `analyze()` to
    /// re-aggregate.
    pub fn aggregate(&mut self) -> Result<String, JsValue> {
        self.inner
            .aggregate()
            .map(|r| format!("{:?}", r.los))
            .map_err(|e| JsValue::from_str(&e))
    }

    /// Run the full HCM Ch.16 pipeline: evaluate every segment with the
    /// Chapter 18 engine, then aggregate (Equations 16-2 through 16-4 and
    /// the Exhibit 16-3 LOS). Returns the facility LOS letter. Throws when
    /// the facility holds a segment added through `add_segment_summary`,
    /// whose Chapter 18 inputs are placeholders — use `aggregate()` there.
    pub fn analyze(&mut self) -> Result<String, JsValue> {
        if self.n_summary_segments > 0 {
            return Err(JsValue::from_str(&format!(
                "{} of {} segments were added as published summaries, which carry no Chapter 18 \
                 inputs to recompute: call aggregate() instead of analyze()",
                self.n_summary_segments,
                self.inner.segments.len()
            )));
        }
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
