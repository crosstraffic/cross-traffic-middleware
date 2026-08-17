use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use transportations_library::hcm::chapter10::freeway_facilities::{
    segment_ramp_section as core_segment_ramp_section, FacilitySegment, FreewayFacility,
    SegmentType, Terrain, RAMP_INFLUENCE_AREA_FT,
};
use transportations_library::hcm::chapter10::managed_lanes::{CrossWeave, ManagedLaneFacility, MlSegmentInput};
use transportations_library::hcm::chapter10::planning::{PlanningFacility, PlanningSection, PlanningSectionType};
use transportations_library::hcm::chapter10::WorkZone;
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

    /// Place a work zone on this segment (HCM Chapter 10, Section 4; Equations
    /// 10-7 through 10-12), from a configuration object matching the serde
    /// schema of the library's `WorkZone` — the shape of the library's own
    /// fixtures, so the `work_zone` object of
    /// `tests/ExampleCases/hcm/FreewayFacilities/case4.json` (Example Problem
    /// 4) passes verbatim:
    ///
    /// ```json
    /// {
    ///   "total_lanes": 3, "open_lanes": 2,
    ///   "soft_barrier": true, "rural": false,
    ///   "lateral_distance_ft": 0.0, "night": false,
    ///   "speed_ratio": 1.0909090909090908, "speed_limit_mi_h": 55.0,
    ///   "total_ramp_density": 1.0, "queue_discharge_drop": 0.131
    /// }
    /// ```
    ///
    /// The work zone is a structured input with ten fields, all of which
    /// enter Equations 10-7 through 10-12, so it arrives as a config object
    /// rather than as ten more trailing constructor arguments, the same
    /// choice `WasmManagedLaneFacility` makes for the other Chapter 10 input
    /// that has no home on a segment. This is a setter rather than an
    /// eighteenth constructor argument so that the seventeen-argument
    /// constructor every existing caller uses keeps its exact signature.
    ///
    /// Every field has a serde default and unknown fields are ignored, so a
    /// misspelled field name falls back to its default rather than throwing —
    /// prefer copying names from the fixture files. The defaults describe a
    /// three-to-two urban daylight closure behind a hard barrier, which is not
    /// a no-op: calling this with `{}` places a real work zone.
    pub fn set_work_zone(&mut self, config: JsValue) -> Result<(), JsValue> {
        let wz: WorkZone = serde_wasm_bindgen::from_value(config)
            .map_err(|e| JsValue::from_str(&format!("invalid work zone configuration: {e}")))?;
        self.inner.work_zone = Some(wz);
        Ok(())
    }

    /// Remove the work zone from this segment, restoring unadjusted capacity
    /// and free-flow speed.
    pub fn clear_work_zone(&mut self) {
        self.inner.work_zone = None;
    }

    pub fn has_work_zone(&self) -> bool {
        self.inner.work_zone.is_some()
    }

    /// Equation 10-7 lane closure severity index, or undefined with no work
    /// zone. `LCSI = 1 / (OR x N_o)`, capped at 2.0 (Exhibit 10-15).
    pub fn work_zone_lcsi(&self) -> Option<f64> {
        self.inner.work_zone.as_ref().map(|wz| wz.lcsi())
    }

    /// Equation 10-11 work zone capacity adjustment factor, or undefined with
    /// no work zone. `non_wz_capacity_pc` is the non-work-zone per-lane
    /// capacity in pc/h/ln, which the facility supplies from Equation 12-6 at
    /// the segment's unadjusted FFS when it runs the analysis; pass it here to
    /// read the factor on its own (2,300 pc/h/ln at the FFS 60 of Example
    /// Problem 4, giving CAF_wz = 0.892).
    pub fn work_zone_caf(&self, non_wz_capacity_pc: f64) -> Option<f64> {
        self.inner.work_zone.as_ref().map(|wz| wz.caf(non_wz_capacity_pc))
    }

    /// Equation 10-12 work zone speed adjustment factor, or undefined with no
    /// work zone. `non_wz_ffs` is the segment free-flow speed in mi/h (60 in
    /// Example Problem 4, giving SAF_wz = 0.982).
    pub fn work_zone_saf(&self, non_wz_ffs: f64) -> Option<f64> {
        self.inner.work_zone.as_ref().map(|wz| wz.saf(non_wz_ffs))
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
    pub(crate) inner: FreewayFacility,
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

    /// Segment capacity, veh/h (Exhibits 25-63 and 25-71). This is the
    /// denominator of `get_dc_ratio()` and it varies by period, both because a
    /// weaving segment's capacity follows the period's weaving pattern and
    /// because the Step A-8 adjustments are per period. Where a work zone is
    /// placed this is the post-CAF_wz value, not the Step A-7 lane-closure
    /// capacity the exhibit prints: Exhibit 25-71 prints 4,499 veh/h for the
    /// Example Problem 4 work zone segment, and the Exhibit 25-72 d/c ratios
    /// of the same problem only reproduce against 4,499 x 0.892.
    pub fn get_capacity(&self, seg: usize, period: usize) -> f64 {
        self.inner.capacity.get(seg).and_then(|r| r.get(period)).copied().unwrap_or(0.0)
    }

    /// Volume served v_a, veh/h (Exhibit 25-48/25-56). This equals the
    /// segment demand only while the facility is undersaturated. Once a
    /// queue forms the oversaturated engine meters what the segment can
    /// actually discharge, so volume served and demand diverge, and it is
    /// volume served that the speed and density of the period are computed
    /// from.
    pub fn get_volume_served(&self, seg: usize, period: usize) -> f64 {
        self.inner.volume_served.get(seg).and_then(|r| r.get(period)).copied().unwrap_or(0.0)
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

    /// Segment capacity `[segment][period]`, veh/h (Exhibits 25-63, 25-71).
    /// See `get_capacity()` for what a work zone segment holds here.
    pub fn capacity_matrix(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.inner.capacity).unwrap_or(JsValue::NULL)
    }

    pub fn los_matrix(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.inner.los).unwrap_or(JsValue::NULL)
    }

    pub fn queue_matrix(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.inner.queue_length_ft).unwrap_or(JsValue::NULL)
    }

    pub fn volume_served_matrix(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.inner.volume_served).unwrap_or(JsValue::NULL)
    }

    /// Demand-based segment LOS `[segment][period]` (Exhibit 25-59, lower
    /// table): "F" where vd/c > 1.0, undefined otherwise. The density-based
    /// `los_matrix()` above reports what the segment delivered at the volume
    /// it served, which can stay at D or E through a period whose demand
    /// exceeded capacity; the demand-based table is where that excess shows
    /// up, so the two are reported side by side rather than merged.
    pub fn demand_based_los_matrix(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.inner.demand_based_los).unwrap_or(JsValue::NULL)
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

/// The managed-lane half of a [`WasmManagedLaneFacility`] built through
/// [`WasmManagedLaneFacility::from_gp`]: the fields of the core's
/// `ManagedLaneFacility` other than `gp`, so the general-purpose lane group
/// can arrive through the positional [`WasmFreewayFacility`] constructor the
/// Chapter 10 callers already use instead of being restated as JSON.
#[derive(Deserialize)]
struct MlConfig {
    #[serde(default)]
    ml: Vec<Option<MlSegmentInput>>,
    #[serde(default)]
    ml_entry_demand: Vec<f64>,
    #[serde(default = "default_ml_ffs")]
    ml_ffs: f64,
    #[serde(default)]
    cross_weave: Vec<Option<CrossWeave>>,
}

/// Matches the core `ManagedLaneFacility::default()` free-flow speed, so a
/// config that omits `ml_ffs` lands where a deserialized facility would.
fn default_ml_ffs() -> f64 {
    60.0
}

/// HCM Chapter 10 managed-lane facility extension (Steps A-9/A-13/A-14/A-17;
/// Chapter 25 Section 2): a general-purpose lane group paired with a parallel
/// managed-lane lane group, analyzed with the cross-weave capacity adjustment
/// on the GP side and the adjacent-friction speed reduction on the ML side,
/// then aggregated per lane group and combined.
///
/// The managed lane is not a segment flag on the GP facility, so it cannot be
/// reached through [`WasmFreewayFacility`]. `ml` is a vector parallel to the
/// GP segments carrying `null` where a GP segment has no adjacent managed
/// lane, and the ML lane group has its own entry demand, free-flow speed, and
/// ramp demands. Those are the inputs this wrapper adds.
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct WasmManagedLaneFacility {
    inner: ManagedLaneFacility,
}

#[wasm_bindgen]
impl WasmManagedLaneFacility {

    /// Build the whole facility, both lane groups, from a configuration
    /// object matching the serde schema of the library's
    /// `ManagedLaneFacility` — the shape of the library's own fixtures, so
    /// `tests/ExampleCases/hcm/FreewayFacilities/ml_case1.json` (Example
    /// Problem 5) loads verbatim:
    ///
    /// ```json
    /// {
    ///   "gp": { "segments": [ ... ], "mainline_demand": [ ... ], "ffs": 60.0 },
    ///   "ml": [ { "lane_type": "ContinuousAccess", "lanes": 1 }, null ],
    ///   "ml_entry_demand": [1000.0, 1100.0],
    ///   "ml_ffs": 60.0
    /// }
    /// ```
    ///
    /// `gp` is the same schema [`WasmFreewayFacility`] takes positionally.
    /// `ml` must have one entry per GP segment once `run_analysis()` is
    /// called, `null` marking a segment with no adjacent managed lane.
    /// `lane_type` is one of `ContinuousAccess` / `Buffer1` / `Buffer2` /
    /// `Barrier1` / `Barrier2` (Exhibit 12-9); only the first two are subject
    /// to the Step A-13 adjacent friction. An ML segment may also carry
    /// `ffs`, `caf`, `saf`, `on_ramp_demand`, and `off_ramp_demand`. The
    /// optional `cross_weave` vector is parallel to the GP segments as well,
    /// each entry `{"cw_demand_pc": [...], "l_cw_min_ft": 0.0}` (Step A-9,
    /// Equations 13-24/13-25); omit it and no cross-weave reduction applies.
    ///
    /// Every field has a serde default and unknown fields are ignored, so a
    /// misspelled field name falls back to its default rather than throwing —
    /// prefer copying names from the fixture files.
    #[wasm_bindgen(constructor)]
    pub fn new(config: JsValue) -> Result<WasmManagedLaneFacility, JsValue> {
        let inner: ManagedLaneFacility = serde_wasm_bindgen::from_value(config)
            .map_err(|e| JsValue::from_str(&format!("invalid managed-lane facility configuration: {e}")))?;
        Ok(WasmManagedLaneFacility { inner })
    }

    /// Build from an already-constructed general-purpose facility plus the
    /// managed-lane half as a config object (`ml`, `ml_entry_demand`,
    /// `ml_ffs`, `cross_weave`, as documented on the constructor). The GP
    /// facility is copied, not consumed, and need not have been run.
    pub fn from_gp(gp: &WasmFreewayFacility, ml_config: JsValue) -> Result<WasmManagedLaneFacility, JsValue> {
        let cfg: MlConfig = serde_wasm_bindgen::from_value(ml_config)
            .map_err(|e| JsValue::from_str(&format!("invalid managed-lane configuration: {e}")))?;
        let mut inner = ManagedLaneFacility::new();
        inner.gp = gp.inner.clone();
        inner.ml = cfg.ml;
        inner.ml_entry_demand = cfg.ml_entry_demand;
        inner.ml_ffs = cfg.ml_ffs;
        inner.cross_weave = cfg.cross_weave;
        Ok(WasmManagedLaneFacility { inner })
    }

    /// Run both lane groups and the combined aggregation. Throws when `ml`
    /// (or a non-empty `cross_weave`) does not have one entry per GP segment,
    /// and on any GP validation failure.
    pub fn run_analysis(&mut self) -> Result<(), JsValue> {
        self.inner.run_analysis().map_err(|e| JsValue::from_str(&e))
    }

    pub fn num_segments(&self) -> usize {
        self.inner.num_segments()
    }

    pub fn num_periods(&self) -> usize {
        self.inner.num_periods()
    }

    /// The general-purpose lane group as a [`WasmFreewayFacility`], which is
    /// how the GP segment matrices are read. This is a snapshot copy taken
    /// after `run_analysis()`, so the Step A-9 cross-weave CAF is already
    /// folded into its segment capacities; running it again is harmless but
    /// pointless.
    pub fn gp_facility(&self) -> WasmFreewayFacility {
        WasmFreewayFacility { inner: self.inner.gp.clone() }
    }

    /// ML segment demand, veh/h — the entry demand accumulated through the ML
    /// ramp demands, not metered by capacity.
    pub fn get_ml_demand(&self, seg: usize, period: usize) -> f64 {
        self.inner.ml_demand.get(seg).and_then(|r| r.get(period)).copied().unwrap_or(0.0)
    }

    /// ML segment capacity, veh/h (Exhibit 25-81): the Chapter 12 adjusted
    /// per-lane capacity times the lane count and the facility heavy-vehicle
    /// factor, so it is a vehicle rate and not the pc/h/ln of Equation 12-14.
    pub fn get_ml_capacity(&self, seg: usize, period: usize) -> f64 {
        self.inner.ml_capacity.get(seg).and_then(|r| r.get(period)).copied().unwrap_or(0.0)
    }

    pub fn get_ml_dc_ratio(&self, seg: usize, period: usize) -> f64 {
        self.inner.ml_dc_ratio.get(seg).and_then(|r| r.get(period)).copied().unwrap_or(0.0)
    }

    pub fn get_ml_speed(&self, seg: usize, period: usize) -> f64 {
        self.inner.ml_speed.get(seg).and_then(|r| r.get(period)).copied().unwrap_or(0.0)
    }

    pub fn get_ml_density_veh(&self, seg: usize, period: usize) -> f64 {
        self.inner.ml_density_veh.get(seg).and_then(|r| r.get(period)).copied().unwrap_or(0.0)
    }

    pub fn get_ml_density_pc(&self, seg: usize, period: usize) -> f64 {
        self.inner.ml_density_pc.get(seg).and_then(|r| r.get(period)).copied().unwrap_or(0.0)
    }

    pub fn get_ml_los(&self, seg: usize, period: usize) -> String {
        self.inner
            .ml_los
            .get(seg)
            .and_then(|r| r.get(period))
            .map(|l| l.to_string())
            .unwrap_or_else(|| "-".to_string())
    }

    /// Whether the Step A-13 adjacent friction was active on the ML segment,
    /// which needs both a friction-capable lane type (continuous access or
    /// Buffer 1) and an adjacent GP density above 35 pc/mi/ln. The speed drop
    /// it causes is already in `get_ml_speed()`; this reports why.
    pub fn is_ml_friction_active(&self, seg: usize, period: usize) -> bool {
        self.inner
            .ml_friction_active
            .get(seg)
            .and_then(|r| r.get(period))
            .copied()
            .unwrap_or(false)
    }

    pub fn get_gp_group_speed(&self, period: usize) -> f64 {
        self.inner.gp_group_performance.get(period).map(|g| g.space_mean_speed).unwrap_or(0.0)
    }

    pub fn get_gp_group_density_veh(&self, period: usize) -> f64 {
        self.inner.gp_group_performance.get(period).map(|g| g.avg_density_veh).unwrap_or(0.0)
    }

    pub fn get_gp_group_los(&self, period: usize) -> String {
        self.inner
            .gp_group_performance
            .get(period)
            .map(|g| g.los.to_string())
            .unwrap_or_else(|| "-".to_string())
    }

    pub fn get_ml_group_speed(&self, period: usize) -> f64 {
        self.inner.ml_group_performance.get(period).map(|g| g.space_mean_speed).unwrap_or(0.0)
    }

    pub fn get_ml_group_density_veh(&self, period: usize) -> f64 {
        self.inner.ml_group_performance.get(period).map(|g| g.avg_density_veh).unwrap_or(0.0)
    }

    pub fn get_ml_group_los(&self, period: usize) -> String {
        self.inner
            .ml_group_performance
            .get(period)
            .map(|g| g.los.to_string())
            .unwrap_or_else(|| "-".to_string())
    }

    pub fn get_facility_speed(&self, period: usize) -> f64 {
        self.inner.facility_performance.get(period).map(|p| p.space_mean_speed).unwrap_or(0.0)
    }

    /// Combined facility density, veh/mi/ln (Exhibit 25-87).
    ///
    /// VERIFY-HCM: this is the exact Equation 10-1 lane-mile-weighted
    /// combination of the two lane-group densities. In Example Problem 5
    /// Period 3 it gives 28.3 where Exhibit 25-87 prints 29.1, a value not
    /// reproducible from the book's own Exhibit 25-86 group densities (31.0
    /// GP, 20.0 ML) under Equation 10-1. LOS is unaffected.
    pub fn get_facility_density_veh(&self, period: usize) -> f64 {
        self.inner.facility_performance.get(period).map(|p| p.avg_density_veh).unwrap_or(0.0)
    }

    pub fn get_facility_los(&self, period: usize) -> String {
        self.inner
            .facility_performance
            .get(period)
            .map(|p| p.los.to_string())
            .unwrap_or_else(|| "-".to_string())
    }

    pub fn ml_capacity_matrix(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.inner.ml_capacity).unwrap_or(JsValue::NULL)
    }

    pub fn ml_dc_matrix(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.inner.ml_dc_ratio).unwrap_or(JsValue::NULL)
    }

    pub fn ml_speed_matrix(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.inner.ml_speed).unwrap_or(JsValue::NULL)
    }

    pub fn ml_density_matrix(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.inner.ml_density_veh).unwrap_or(JsValue::NULL)
    }

    pub fn ml_los_matrix(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.inner.ml_los).unwrap_or(JsValue::NULL)
    }

    pub fn ml_friction_matrix(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.inner.ml_friction_active).unwrap_or(JsValue::NULL)
    }

    /// Both lane groups by period (Exhibit 25-86): space mean speed, average
    /// density in veh/mi/ln and pc/mi/ln, and LOS.
    pub fn lane_group_performance_to_js_value(&self) -> JsValue {
        let obj = js_sys::Object::new();
        js_sys::Reflect::set(&obj, &JsValue::from_str("gp"), &serde_wasm_bindgen::to_value(&self.inner.gp_group_performance).unwrap_or(JsValue::NULL)).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("ml"), &serde_wasm_bindgen::to_value(&self.inner.ml_group_performance).unwrap_or(JsValue::NULL)).unwrap();
        JsValue::from(obj)
    }

    pub fn results_to_js_value(&self) -> JsValue {
        let periods = self.inner.num_periods();
        let facility_speed: Vec<f64> = (0..periods).map(|p| self.get_facility_speed(p)).collect();
        let facility_density_veh: Vec<f64> = (0..periods).map(|p| self.get_facility_density_veh(p)).collect();
        let facility_los: Vec<String> = (0..periods).map(|p| self.get_facility_los(p)).collect();

        let obj = js_sys::Object::new();
        js_sys::Reflect::set(&obj, &JsValue::from_str("num_segments"), &JsValue::from(self.num_segments() as u32)).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("num_periods"), &JsValue::from(periods as u32)).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("facility_speed"), &serde_wasm_bindgen::to_value(&facility_speed).unwrap_or(JsValue::NULL)).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("facility_density_veh"), &serde_wasm_bindgen::to_value(&facility_density_veh).unwrap_or(JsValue::NULL)).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("facility_los"), &serde_wasm_bindgen::to_value(&facility_los).unwrap_or(JsValue::NULL)).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("lane_groups"), &self.lane_group_performance_to_js_value()).unwrap();

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

    /// Section delay rate ΔR, s/mi (Exhibit 25-92). Only the undersaturated
    /// term ΔRU of Equation 25-47 is reported, evaluated at the actual d/c
    /// even above 1.0; the library reproduces the worked Example Problem 6
    /// rather than the ΔRU + ΔRO form of Equation 25-49, and expresses
    /// oversaturation through the vertical queue instead (see the VERIFY-HCM
    /// note in the library's `planning.rs`).
    pub fn get_delay_rate(&self, section: usize, period: usize) -> f64 {
        self.inner
            .section_results
            .get(section)
            .and_then(|r| r.get(period))
            .map(|r| r.delay_rate)
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

    /// Total vertical-queue length across all sections at the end of the
    /// period, mi (Exhibit 25-96). This is the planning method's only
    /// account of oversaturation, since its delay and travel rates leave
    /// the ΔRO term out (see `get_delay_rate`).
    pub fn get_facility_queue_mi(&self, period: usize) -> f64 {
        self.inner
            .facility_results
            .get(period)
            .map(|r| r.total_queue_mi)
            .unwrap_or(0.0)
    }

    pub fn results_to_js_value(&self) -> JsValue {
        let periods = self.inner.facility_results.len();
        let speed: Vec<f64> = (0..periods).map(|p| self.get_facility_speed(p)).collect();
        let density: Vec<f64> = (0..periods).map(|p| self.get_facility_density(p)).collect();
        let los: Vec<String> = (0..periods).map(|p| self.get_facility_los(p)).collect();
        let travel_time_min: Vec<f64> = self.inner.facility_results.iter().map(|r| r.travel_time_min).collect();
        let oversaturated: Vec<bool> = self.inner.facility_results.iter().map(|r| r.oversaturated).collect();
        let total_queue_mi: Vec<f64> = self.inner.facility_results.iter().map(|r| r.total_queue_mi).collect();

        let obj = js_sys::Object::new();
        js_sys::Reflect::set(&obj, &JsValue::from_str("num_sections"), &JsValue::from(self.num_sections() as u32)).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("total_length_mi"), &JsValue::from(self.total_length_mi())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("facility_speed"), &serde_wasm_bindgen::to_value(&speed).unwrap_or(JsValue::NULL)).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("facility_density"), &serde_wasm_bindgen::to_value(&density).unwrap_or(JsValue::NULL)).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("facility_los"), &serde_wasm_bindgen::to_value(&los).unwrap_or(JsValue::NULL)).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("travel_time_min"), &serde_wasm_bindgen::to_value(&travel_time_min).unwrap_or(JsValue::NULL)).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("oversaturated"), &serde_wasm_bindgen::to_value(&oversaturated).unwrap_or(JsValue::NULL)).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("total_queue_mi"), &serde_wasm_bindgen::to_value(&total_queue_mi).unwrap_or(JsValue::NULL)).unwrap();

        JsValue::from(obj)
    }
}

// ═════════════════════════════════════════════════════════════════════════
// Segmentation rules (Step A-2)
// ═════════════════════════════════════════════════════════════════════════

/// One piece of a segmented ramp section. The key names are the fixture
/// schema's, so a returned piece is already the head of a `FacilitySegment`
/// and a caller assembling a facility adds lanes and demands to it rather
/// than translating it.
#[derive(Serialize)]
pub struct RampSectionPiece {
    pub seg_type: String,
    pub length_ft: f64,
}

/// HCM Chapter 10 Step A-2: divide the section between an on-ramp gore and
/// the next off-ramp gore into analysis segments, per the segmentation rules
/// of Section 2 and Exhibits 10-11 and 10-12.
///
/// Returns `[{ seg_type, length_ft }]` in upstream-to-downstream order, with
/// zero-length pieces omitted:
///
/// - auxiliary lane between the gores: one `Weaving` piece;
/// - gore-to-gore above 3,000 ft: `Merge` 1,500 + `Basic` (spacing − 3,000)
///   + `Diverge` 1,500;
/// - 1,500 ft to 3,000 ft: `Merge` (spacing − 1,500) + `OverlappingRamp`
///   (3,000 − spacing) + `Diverge` (spacing − 1,500);
/// - 1,500 ft or less with no auxiliary lane: a single `OverlappingRamp`
///   over the whole distance, the truncation the manual calls highly
///   unusual.
///
/// The one thing to read twice is what `gore_to_gore_ft` means in the
/// auxiliary-lane case, because the answer comes back as the caller sent it
/// and so a wrong value is invisible. The weaving *segment* is not the
/// gore-to-gore distance: Chapter 10's segmentation rules put its boundaries
/// 500 ft upstream and 500 ft downstream of the two gores (Exhibit 10-2), so
/// a caller placing ramps by gore station must pass gore-to-gore + 1,000 ft
/// here and carry the gore-to-gore distance itself as the segment's
/// `short_length_ft`. Example Problem 1 is the check: its weaving segment is
/// 2,640 ft long with a 1,640 ft short length.
#[wasm_bindgen]
pub fn segment_ramp_section(
    gore_to_gore_ft: f64,
    has_auxiliary_lane: bool,
) -> Result<JsValue, JsValue> {
    validate_gore_to_gore(gore_to_gore_ft).map_err(|e| JsValue::from_str(&e))?;
    let pieces: Vec<RampSectionPiece> = core_segment_ramp_section(gore_to_gore_ft, has_auxiliary_lane)
        .into_iter()
        .map(|(seg_type, length_ft)| RampSectionPiece {
            seg_type: format!("{seg_type:?}"),
            length_ft,
        })
        .collect();
    serde_wasm_bindgen::to_value(&pieces).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Ramp influence area length, ft: 1,500 ft downstream of an on-ramp gore
/// and upstream of an off-ramp gore (Chapter 10 Section 2; Exhibit 10-1).
///
/// Bound so that a caller placing an isolated ramp — one with no paired ramp
/// downstream of it, which `segment_ramp_section` does not describe — reads
/// the length of its merge or diverge segment from the library rather than
/// writing 1,500 down a second time.
#[wasm_bindgen]
pub fn ramp_influence_area_ft() -> f64 {
    RAMP_INFLUENCE_AREA_FT
}

/// The guard is for the NaN, and the NaN is dangerous here rather than merely
/// wrong: every comparison in the core's branch chain is false against it, so
/// the section falls through to the single-overlap arm and comes back as one
/// well-formed `OverlappingRamp` piece of NaN length. That length then passes
/// `FreewayFacility::validate`, whose test is `length_ft <= 0.0`, and a
/// facility built on it analyzes and prints numbers. A non-positive spacing is
/// caught by that same test downstream, but it is rejected here too, because
/// the section it describes does not exist.
fn validate_gore_to_gore(gore_to_gore_ft: f64) -> Result<(), String> {
    if !gore_to_gore_ft.is_finite() || gore_to_gore_ft <= 0.0 {
        return Err(format!(
            "gore_to_gore_ft must be finite and positive, got {gore_to_gore_ft}"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// What the guard is for. Without it the core hands back a piece that is
    /// well-formed in every way a downstream check looks at, so the assertion
    /// is on the core's own return rather than only on the rejection.
    #[test]
    fn a_non_finite_spacing_would_return_a_nan_length_segment() {
        let pieces = core_segment_ramp_section(f64::NAN, false);
        assert_eq!(pieces.len(), 1, "the branch chain falls through to the overlap arm");
        assert_eq!(pieces[0].0, SegmentType::OverlappingRamp);
        assert!(pieces[0].1.is_nan());
        // And that length survives the facility validator, whose length test
        // is `<= 0.0` and is therefore false against a NaN.
        // The validator's expression is reproduced literally rather than
        // rewritten, because the point is that this exact test is what lets a
        // NaN through.
        #[allow(clippy::neg_cmp_op_on_partial_ord)]
        {
            assert!(!(pieces[0].1 <= 0.0));
        }

        assert!(validate_gore_to_gore(f64::NAN).is_err());
        assert!(validate_gore_to_gore(f64::INFINITY).is_err());
        assert!(validate_gore_to_gore(0.0).is_err());
        assert!(validate_gore_to_gore(-1500.0).is_err());
        // The control: a real spacing passes and still segments the way
        // Exhibit 10-11 says it does.
        validate_gore_to_gore(2000.0).expect("a real spacing must pass the guard");
        assert_eq!(
            core_segment_ramp_section(2000.0, false),
            vec![
                (SegmentType::Merge, 500.0),
                (SegmentType::OverlappingRamp, 1000.0),
                (SegmentType::Diverge, 500.0),
            ]
        );
    }

    /// The influence-area constant is bound rather than restated, so the test
    /// that matters is that the binding reads the library's value.
    #[test]
    fn the_influence_area_is_the_librarys_constant() {
        assert_eq!(ramp_influence_area_ft(), RAMP_INFLUENCE_AREA_FT);
        assert_eq!(ramp_influence_area_ft(), 1500.0);
    }
}
