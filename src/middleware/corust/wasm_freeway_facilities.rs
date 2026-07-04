use wasm_bindgen::prelude::*;
use transportations_library::hcm::chapter10::freeway_facilities::{FacilitySegment, FreewayFacility, SegmentType, Terrain};
use transportations_library::hcm::chapter10::planning::{PlanningFacility, PlanningSection, PlanningSectionType};
use transportations_library::hcm::common::CityType;

pub(crate) fn parse_terrain(s: &str) -> Terrain {
    match s.to_lowercase().as_str() {
        "rolling" => Terrain::Rolling,
        "mountainous" => Terrain::Mountainous,
        _ => Terrain::Level,
    }
}

pub(crate) fn parse_city_type(s: &str) -> CityType {
    match s.to_lowercase().as_str() {
        "rural" => CityType::Rural,
        _ => CityType::Urban,
    }
}

fn parse_seg_type(s: &str) -> SegmentType {
    match s.to_lowercase().replace([' ', '_', '-'], "").as_str() {
        "merge" | "onramp" => SegmentType::Merge,
        "diverge" | "offramp" => SegmentType::Diverge,
        "weaving" | "weave" => SegmentType::Weaving,
        "overlappingramp" | "overlapping" | "rampoverlap" => SegmentType::OverlappingRamp,
        _ => SegmentType::Basic,
    }
}

/// One HCM Chapter 10 analysis segment (Basic / Merge / Diverge / Weaving /
/// OverlappingRamp). Ramp demand vectors carry one value per 15-min analysis
/// period, veh/h.
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct WasmFacilitySegment {
    pub(crate) inner: FacilitySegment,
}

#[wasm_bindgen]
impl WasmFacilitySegment {

    #[wasm_bindgen(constructor)]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        seg_type: String,
        length_ft: f64,
        lanes: u32,
        on_ramp_demand: Vec<f64>,
        off_ramp_demand: Vec<f64>,
        ramp_to_ramp_demand: Vec<f64>,
        ramp_ffs: Option<f64>,
        accel_lane_ft: Option<f64>,
        decel_lane_ft: Option<f64>,
        short_length_ft: Option<f64>,
        num_weaving_lanes: Option<u32>,
        lc_rf: Option<u32>,
        lc_fr: Option<u32>,
        ffs: Option<f64>,
        caf: Option<f64>,
        saf: Option<f64>,
        daf: Option<f64>,
    ) -> Self {
        let mut inner = FacilitySegment::default();
        inner.seg_type = parse_seg_type(&seg_type);
        inner.length_ft = length_ft;
        inner.lanes = lanes;
        inner.on_ramp_demand = on_ramp_demand;
        inner.off_ramp_demand = off_ramp_demand;
        inner.ramp_to_ramp_demand = ramp_to_ramp_demand;
        if let Some(v) = ramp_ffs {
            inner.ramp_ffs = v;
        }
        if let Some(v) = accel_lane_ft {
            inner.accel_lane_ft = v;
        }
        if let Some(v) = decel_lane_ft {
            inner.decel_lane_ft = v;
        }
        if short_length_ft.is_some() {
            inner.short_length_ft = short_length_ft;
        }
        if let Some(v) = num_weaving_lanes {
            inner.num_weaving_lanes = v;
        }
        if let Some(v) = lc_rf {
            inner.lc_rf = v;
        }
        if let Some(v) = lc_fr {
            inner.lc_fr = v;
        }
        if ffs.is_some() {
            inner.ffs = ffs;
        }
        if let Some(v) = caf {
            inner.caf = v;
        }
        if let Some(v) = saf {
            inner.saf = v;
        }
        if let Some(v) = daf {
            inner.daf = v;
        }
        WasmFacilitySegment { inner }
    }

    pub fn get_seg_type(&self) -> String {
        format!("{:?}", self.inner.seg_type)
    }

    pub fn get_length_ft(&self) -> f64 {
        self.inner.length_ft
    }

    pub fn get_lanes(&self) -> u32 {
        self.inner.lanes
    }
}

/// Build the core FreewayFacility from segment wrappers and global inputs.
/// Shared with the Chapter 11 reliability binding.
#[allow(clippy::too_many_arguments)]
pub(crate) fn build_facility(
    wasm_segments: &[WasmFacilitySegment],
    mainline_demand: Vec<f64>,
    ffs: Option<f64>,
    heavy_vehicle_pct: Option<f64>,
    terrain: Option<String>,
    city_type: Option<String>,
    phf: Option<f64>,
    jam_density_pc: Option<f64>,
    queue_discharge_drop: Option<f64>,
    total_ramp_density: Option<f64>,
    interchange_density: Option<f64>,
) -> FreewayFacility {
    let mut inner = FreewayFacility::new();
    inner.segments = wasm_segments.iter().map(|s| s.inner.clone()).collect();
    inner.mainline_demand = mainline_demand;
    if let Some(v) = ffs {
        inner.ffs = v;
    }
    if let Some(v) = heavy_vehicle_pct {
        inner.heavy_vehicle_pct = v;
    }
    if let Some(t) = terrain {
        inner.terrain = parse_terrain(&t);
    }
    if let Some(ct) = city_type {
        inner.city_type = parse_city_type(&ct);
    }
    if let Some(v) = phf {
        inner.phf = v;
    }
    if let Some(v) = jam_density_pc {
        inner.jam_density_pc = v;
    }
    if let Some(v) = queue_discharge_drop {
        inner.queue_discharge_drop = v;
    }
    if let Some(v) = total_ramp_density {
        inner.total_ramp_density = v;
    }
    if interchange_density.is_some() {
        inner.interchange_density = interchange_density;
    }
    inner
}

/// HCM Chapter 10 freeway facilities core methodology (Steps A-1 through
/// A-17): a directional facility of ordered segments evaluated over
/// consecutive 15-min analysis periods.
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct WasmFreewayFacility {
    inner: FreewayFacility,
}

#[wasm_bindgen]
impl WasmFreewayFacility {

    #[wasm_bindgen(constructor)]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        wasm_segments: Vec<WasmFacilitySegment>,
        mainline_demand: Vec<f64>,
        ffs: Option<f64>,
        heavy_vehicle_pct: Option<f64>,
        terrain: Option<String>,
        city_type: Option<String>,
        phf: Option<f64>,
        jam_density_pc: Option<f64>,
        queue_discharge_drop: Option<f64>,
        total_ramp_density: Option<f64>,
        interchange_density: Option<f64>,
    ) -> Self {
        WasmFreewayFacility {
            inner: build_facility(
                &wasm_segments,
                mainline_demand,
                ffs,
                heavy_vehicle_pct,
                terrain,
                city_type,
                phf,
                jam_density_pc,
                queue_discharge_drop,
                total_ramp_density,
                interchange_density,
            ),
        }
    }

    /// Run the full core methodology. Throws with the validation message on
    /// invalid input (e.g. first/last segment not basic, no periods).
    pub fn run_analysis(&mut self) -> Result<(), JsValue> {
        self.inner.run_analysis().map_err(|e| JsValue::from_str(&e))
    }

    pub fn num_segments(&self) -> usize {
        self.inner.num_segments()
    }

    pub fn num_periods(&self) -> usize {
        self.inner.num_periods()
    }

    pub fn total_length_mi(&self) -> f64 {
        self.inner.total_length_mi()
    }

    pub fn is_oversaturated(&self) -> bool {
        self.inner.oversaturated
    }

    pub fn get_speed(&self, seg: usize, period: usize) -> f64 {
        self.inner.speed.get(seg).and_then(|r| r.get(period)).copied().unwrap_or(0.0)
    }

    pub fn get_density_veh(&self, seg: usize, period: usize) -> f64 {
        self.inner.density_veh.get(seg).and_then(|r| r.get(period)).copied().unwrap_or(0.0)
    }

    pub fn get_density_pc(&self, seg: usize, period: usize) -> f64 {
        self.inner.density_pc.get(seg).and_then(|r| r.get(period)).copied().unwrap_or(0.0)
    }

    pub fn get_dc_ratio(&self, seg: usize, period: usize) -> f64 {
        self.inner.dc_ratio.get(seg).and_then(|r| r.get(period)).copied().unwrap_or(0.0)
    }

    pub fn get_queue_length_ft(&self, seg: usize, period: usize) -> f64 {
        self.inner.queue_length_ft.get(seg).and_then(|r| r.get(period)).copied().unwrap_or(0.0)
    }

    pub fn get_los(&self, seg: usize, period: usize) -> String {
        self.inner
            .los
            .get(seg)
            .and_then(|r| r.get(period))
            .map(|l| l.to_string())
            .unwrap_or_else(|| "-".to_string())
    }

    pub fn get_facility_speed(&self, period: usize) -> f64 {
        self.inner
            .facility_performance
            .get(period)
            .map(|p| p.space_mean_speed)
            .unwrap_or(0.0)
    }

    pub fn get_facility_density_veh(&self, period: usize) -> f64 {
        self.inner
            .facility_performance
            .get(period)
            .map(|p| p.avg_density_veh)
            .unwrap_or(0.0)
    }

    pub fn get_facility_los(&self, period: usize) -> String {
        self.inner
            .facility_performance
            .get(period)
            .map(|p| p.los.to_string())
            .unwrap_or_else(|| "-".to_string())
    }

    pub fn get_overall_speed(&self) -> f64 {
        self.inner.overall_space_mean_speed()
    }

    pub fn get_overall_density_veh(&self) -> f64 {
        self.inner.overall_density_veh()
    }

    pub fn speed_matrix(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.inner.speed).unwrap_or(JsValue::NULL)
    }

    pub fn density_matrix(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.inner.density_veh).unwrap_or(JsValue::NULL)
    }

    pub fn dc_matrix(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.inner.dc_ratio).unwrap_or(JsValue::NULL)
    }

    pub fn los_matrix(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.inner.los).unwrap_or(JsValue::NULL)
    }

    pub fn queue_matrix(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.inner.queue_length_ft).unwrap_or(JsValue::NULL)
    }

    pub fn results_to_js_value(&self) -> JsValue {
        let periods = self.inner.num_periods();
        let facility_speed: Vec<f64> = (0..periods).map(|p| self.get_facility_speed(p)).collect();
        let facility_density_veh: Vec<f64> = (0..periods).map(|p| self.get_facility_density_veh(p)).collect();
        let facility_los: Vec<String> = (0..periods).map(|p| self.get_facility_los(p)).collect();

        let obj = js_sys::Object::new();
        js_sys::Reflect::set(&obj, &JsValue::from_str("num_segments"), &JsValue::from(self.num_segments() as u32)).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("num_periods"), &JsValue::from(periods as u32)).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("total_length_mi"), &JsValue::from(self.total_length_mi())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("oversaturated"), &JsValue::from(self.is_oversaturated())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("facility_speed"), &serde_wasm_bindgen::to_value(&facility_speed).unwrap_or(JsValue::NULL)).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("facility_density_veh"), &serde_wasm_bindgen::to_value(&facility_density_veh).unwrap_or(JsValue::NULL)).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("facility_los"), &serde_wasm_bindgen::to_value(&facility_los).unwrap_or(JsValue::NULL)).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("overall_speed"), &JsValue::from(self.get_overall_speed())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("overall_density_veh"), &JsValue::from(self.get_overall_density_veh())).unwrap();

        JsValue::from(obj)
    }
}

/// HCM Chapter 25, Section 6 planning-level freeway facility method (the
/// screening companion to the Chapter 10 core methodology). Sections are
/// passed as parallel arrays; `sec_types` is a comma-separated list of
/// "basic", "ramp", or "weave" entries.
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct WasmPlanningFacility {
    inner: PlanningFacility,
}

#[wasm_bindgen]
impl WasmPlanningFacility {

    #[wasm_bindgen(constructor)]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        sec_types: String,
        lengths_mi: Vec<f64>,
        lanes: Vec<u32>,
        inflow_aadt: Vec<f64>,
        outflow_aadt: Vec<f64>,
        weave_vr: Vec<f64>,
        ffs: Option<f64>,
        k_factor: Option<f64>,
        growth_factor: Option<f64>,
        phf: Option<f64>,
        pct_sut: Option<f64>,
        pct_tt: Option<f64>,
        terrain: Option<String>,
        city_type: Option<String>,
    ) -> Self {
        let types: Vec<PlanningSectionType> = sec_types
            .split(',')
            .map(|t| match t.trim().to_lowercase().as_str() {
                "ramp" => PlanningSectionType::Ramp,
                "weave" | "weaving" => PlanningSectionType::Weave,
                _ => PlanningSectionType::Basic,
            })
            .collect();

        let mut inner = PlanningFacility::new();
        inner.sections = lengths_mi
            .iter()
            .enumerate()
            .map(|(i, &length)| {
                let mut sec = PlanningSection::default();
                sec.sec_type = types.get(i).copied().unwrap_or(PlanningSectionType::Basic);
                sec.length_mi = length;
                sec.lanes = lanes.get(i).copied().unwrap_or(3);
                sec.inflow_aadt = inflow_aadt.get(i).copied().unwrap_or(0.0);
                sec.outflow_aadt = outflow_aadt.get(i).copied().unwrap_or(0.0);
                sec.weave_vr = weave_vr.get(i).copied().unwrap_or(0.0);
                sec
            })
            .collect();
        if let Some(v) = ffs {
            inner.ffs = v;
        }
        if let Some(v) = k_factor {
            inner.k_factor = v;
        }
        if let Some(v) = growth_factor {
            inner.growth_factor = v;
        }
        if let Some(v) = phf {
            inner.phf = v;
        }
        if let Some(v) = pct_sut {
            inner.pct_sut = v;
        }
        if let Some(v) = pct_tt {
            inner.pct_tt = v;
        }
        if let Some(t) = terrain {
            inner.terrain = parse_terrain(&t);
        }
        if let Some(ct) = city_type {
            inner.city_type = parse_city_type(&ct);
        }
        WasmPlanningFacility { inner }
    }

    /// Run the planning-level analysis (Steps 1-5, four 15-min periods).
    pub fn run_analysis(&mut self) -> Result<(), JsValue> {
        self.inner.run_analysis().map_err(|e| JsValue::from_str(&e))
    }

    pub fn num_sections(&self) -> usize {
        self.inner.num_sections()
    }

    pub fn total_length_mi(&self) -> f64 {
        self.inner.total_length_mi()
    }

    pub fn get_dc_ratio(&self, section: usize, period: usize) -> f64 {
        self.inner
            .section_results
            .get(section)
            .and_then(|r| r.get(period))
            .map(|r| r.dc_ratio)
            .unwrap_or(0.0)
    }

    pub fn get_section_speed(&self, section: usize, period: usize) -> f64 {
        self.inner
            .section_results
            .get(section)
            .and_then(|r| r.get(period))
            .map(|r| r.speed)
            .unwrap_or(0.0)
    }

    pub fn get_section_density(&self, section: usize, period: usize) -> f64 {
        self.inner
            .section_results
            .get(section)
            .and_then(|r| r.get(period))
            .map(|r| r.density)
            .unwrap_or(0.0)
    }

    pub fn get_facility_speed(&self, period: usize) -> f64 {
        self.inner
            .facility_results
            .get(period)
            .map(|r| r.space_mean_speed)
            .unwrap_or(0.0)
    }

    pub fn get_facility_density(&self, period: usize) -> f64 {
        self.inner
            .facility_results
            .get(period)
            .map(|r| r.avg_density)
            .unwrap_or(0.0)
    }

    pub fn get_facility_los(&self, period: usize) -> String {
        self.inner
            .facility_results
            .get(period)
            .map(|r| r.los.to_string())
            .unwrap_or_else(|| "-".to_string())
    }

    pub fn results_to_js_value(&self) -> JsValue {
        let periods = self.inner.facility_results.len();
        let speed: Vec<f64> = (0..periods).map(|p| self.get_facility_speed(p)).collect();
        let density: Vec<f64> = (0..periods).map(|p| self.get_facility_density(p)).collect();
        let los: Vec<String> = (0..periods).map(|p| self.get_facility_los(p)).collect();
        let travel_time_min: Vec<f64> = self.inner.facility_results.iter().map(|r| r.travel_time_min).collect();
        let oversaturated: Vec<bool> = self.inner.facility_results.iter().map(|r| r.oversaturated).collect();

        let obj = js_sys::Object::new();
        js_sys::Reflect::set(&obj, &JsValue::from_str("num_sections"), &JsValue::from(self.num_sections() as u32)).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("total_length_mi"), &JsValue::from(self.total_length_mi())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("facility_speed"), &serde_wasm_bindgen::to_value(&speed).unwrap_or(JsValue::NULL)).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("facility_density"), &serde_wasm_bindgen::to_value(&density).unwrap_or(JsValue::NULL)).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("facility_los"), &serde_wasm_bindgen::to_value(&los).unwrap_or(JsValue::NULL)).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("travel_time_min"), &serde_wasm_bindgen::to_value(&travel_time_min).unwrap_or(JsValue::NULL)).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("oversaturated"), &serde_wasm_bindgen::to_value(&oversaturated).unwrap_or(JsValue::NULL)).unwrap();

        JsValue::from(obj)
    }
}
