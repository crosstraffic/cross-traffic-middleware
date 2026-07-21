use wasm_bindgen::prelude::*;
use transportations_library::twsc::{
    MajorRightTurnConfig, MinorLaneConfig, Mv, Twsc, TwscDemand, TwscGeometry, TwscLaneResult,
    UTurnMedianWidth, ALL_MOVEMENTS,
};

/// HCM Exhibit 20-1 movement labels in the internal computation order.
const MOVEMENT_LABELS: [&str; 14] = [
    "1", "1U", "2", "3", "4", "4U", "5", "6", "7", "8", "9", "10", "11", "12",
];

fn parse_movement(label: &str) -> Result<Mv, JsValue> {
    Mv::from_label(label)
        .ok_or_else(|| JsValue::from_str(&format!("unknown TWSC movement label: {label}")))
}

fn parse_major_right_turn(s: &str) -> Result<MajorRightTurnConfig, JsValue> {
    match s.to_lowercase().as_str() {
        "shared" => Ok(MajorRightTurnConfig::Shared),
        "exclusive" => Ok(MajorRightTurnConfig::Exclusive),
        "channelized" => Ok(MajorRightTurnConfig::Channelized),
        other => Err(JsValue::from_str(&format!(
            "major right-turn configuration must be shared, exclusive, or channelized, got {other}"
        ))),
    }
}

fn parse_minor_lanes(s: &str) -> Result<MinorLaneConfig, JsValue> {
    match s.to_lowercase().as_str() {
        "single_shared" | "singleshared" => Ok(MinorLaneConfig::SingleShared),
        "shared_lt_exclusive_r" | "sharedleftthroughexclusiveright" => {
            Ok(MinorLaneConfig::SharedLeftThroughExclusiveRight)
        }
        "exclusive_l_shared_tr" | "exclusiveleftsharedthroughright" => {
            Ok(MinorLaneConfig::ExclusiveLeftSharedThroughRight)
        }
        "separate" => Ok(MinorLaneConfig::Separate),
        other => Err(JsValue::from_str(&format!(
            "minor lane configuration must be single_shared, shared_lt_exclusive_r, exclusive_l_shared_tr, or separate, got {other}"
        ))),
    }
}

fn mv_label(mv: Mv) -> &'static str {
    MOVEMENT_LABELS[mv.idx()]
}

fn opt_num(v: Option<f64>) -> JsValue {
    v.map(JsValue::from).unwrap_or(JsValue::NULL)
}

fn opt_char(v: Option<char>) -> JsValue {
    v.map(|c| JsValue::from_str(&c.to_string()))
        .unwrap_or(JsValue::NULL)
}

fn set(obj: &js_sys::Object, key: &str, val: JsValue) {
    js_sys::Reflect::set(obj, &JsValue::from_str(key), &val).unwrap();
}

fn lane_to_js_value(l: &TwscLaneResult) -> JsValue {
    let obj = js_sys::Object::new();
    let labels: Vec<String> = l.movements.iter().map(|&m| mv_label(m).to_string()).collect();
    set(&obj, "movements", JsValue::from_str(&labels.join("+")));
    set(&obj, "flow_rate", JsValue::from(l.flow_rate));
    set(&obj, "capacity", JsValue::from(l.capacity));
    set(&obj, "control_delay", JsValue::from(l.control_delay));
    set(&obj, "los", JsValue::from_str(&l.los.to_string()));
    set(&obj, "queue_95", JsValue::from(l.queue_95));
    JsValue::from(obj)
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct WasmTwsc {
    inner: Twsc,
}

#[wasm_bindgen]
impl WasmTwsc {

    #[wasm_bindgen(constructor)]
    pub fn new(
        v1: Option<f64>,
        v1u: Option<f64>,
        v2: Option<f64>,
        v3: Option<f64>,
        v4: Option<f64>,
        v4u: Option<f64>,
        v5: Option<f64>,
        v6: Option<f64>,
        v7: Option<f64>,
        v8: Option<f64>,
        v9: Option<f64>,
        v10: Option<f64>,
        v11: Option<f64>,
        v12: Option<f64>,
        v13_ped: Option<f64>,
        v14_ped: Option<f64>,
        v15_ped: Option<f64>,
        v16_ped: Option<f64>,
        is_three_leg: Option<bool>,
        major_lanes_per_direction: Option<u32>,
        major_right_turn_eb: Option<String>,
        major_right_turn_wb: Option<String>,
        uturn_median_width: Option<String>,
        grade_minor_nb_pct: Option<f64>,
        grade_minor_sb_pct: Option<f64>,
        minor_lanes_nb: Option<String>,
        minor_lanes_sb: Option<String>,
        median_storage_nb: Option<u32>,
        median_storage_sb: Option<u32>,
        flare_storage_nb: Option<u32>,
        flare_storage_sb: Option<u32>,
        lane_width_ft: Option<f64>,
        phf: Option<f64>,
        analysis_period_h: Option<f64>,
        heavy_vehicle_pct: Option<f64>,
    ) -> Result<WasmTwsc, JsValue> {
        let mut demand = TwscDemand::default();
        if let Some(v) = v1 {
            demand.v1 = v;
        }
        if let Some(v) = v1u {
            demand.v1u = v;
        }
        if let Some(v) = v2 {
            demand.v2 = v;
        }
        if let Some(v) = v3 {
            demand.v3 = v;
        }
        if let Some(v) = v4 {
            demand.v4 = v;
        }
        if let Some(v) = v4u {
            demand.v4u = v;
        }
        if let Some(v) = v5 {
            demand.v5 = v;
        }
        if let Some(v) = v6 {
            demand.v6 = v;
        }
        if let Some(v) = v7 {
            demand.v7 = v;
        }
        if let Some(v) = v8 {
            demand.v8 = v;
        }
        if let Some(v) = v9 {
            demand.v9 = v;
        }
        if let Some(v) = v10 {
            demand.v10 = v;
        }
        if let Some(v) = v11 {
            demand.v11 = v;
        }
        if let Some(v) = v12 {
            demand.v12 = v;
        }
        if let Some(v) = v13_ped {
            demand.v13 = v;
        }
        if let Some(v) = v14_ped {
            demand.v14 = v;
        }
        if let Some(v) = v15_ped {
            demand.v15 = v;
        }
        if let Some(v) = v16_ped {
            demand.v16 = v;
        }

        let mut geometry = TwscGeometry::default();
        if let Some(v) = is_three_leg {
            geometry.is_three_leg = v;
        }
        if let Some(v) = major_lanes_per_direction {
            geometry.major_lanes_per_direction = v;
        }
        if let Some(s) = major_right_turn_eb {
            geometry.major_right_turn_eb = parse_major_right_turn(&s)?;
        }
        if let Some(s) = major_right_turn_wb {
            geometry.major_right_turn_wb = parse_major_right_turn(&s)?;
        }
        if let Some(s) = uturn_median_width {
            geometry.uturn_median_width = match s.to_lowercase().as_str() {
                "wide" => UTurnMedianWidth::Wide,
                "narrow" => UTurnMedianWidth::Narrow,
                other => {
                    return Err(JsValue::from_str(&format!(
                        "U-turn median width must be wide or narrow, got {other}"
                    )))
                }
            };
        }
        if let Some(v) = grade_minor_nb_pct {
            geometry.grade_minor_nb_pct = v;
        }
        if let Some(v) = grade_minor_sb_pct {
            geometry.grade_minor_sb_pct = v;
        }
        if let Some(s) = minor_lanes_nb {
            geometry.minor_lanes_nb = parse_minor_lanes(&s)?;
        }
        if let Some(s) = minor_lanes_sb {
            geometry.minor_lanes_sb = parse_minor_lanes(&s)?;
        }
        if median_storage_nb.is_some() {
            geometry.median_storage_nb = median_storage_nb;
        }
        if median_storage_sb.is_some() {
            geometry.median_storage_sb = median_storage_sb;
        }
        if flare_storage_nb.is_some() {
            geometry.flare_storage_nb = flare_storage_nb;
        }
        if flare_storage_sb.is_some() {
            geometry.flare_storage_sb = flare_storage_sb;
        }
        if let Some(v) = lane_width_ft {
            geometry.lane_width_ft = v;
        }

        let mut inner = Twsc::new(demand, geometry);
        if phf.is_some() {
            inner.phf = phf;
        }
        if let Some(v) = analysis_period_h {
            inner.analysis_period_h = v;
        }
        if let Some(v) = heavy_vehicle_pct {
            inner.heavy_vehicle_pct = v;
        }
        Ok(WasmTwsc { inner })
    }

    /// Run the complete HCM Chapter 20 procedure (Steps 1-13).
    pub fn analyze(&mut self) {
        self.inner.analyze();
    }

    /// Demand flow rate of a movement ("1", "1U", ..., "12"), veh/h.
    pub fn get_flow_rate(&self, movement: &str) -> Result<f64, JsValue> {
        Ok(self.inner.get_flow_rate(parse_movement(movement)?))
    }

    /// Conflicting flow rate v_c,x of a movement, veh/h (Step 3).
    pub fn get_conflicting_flow(&self, movement: &str) -> Result<Option<f64>, JsValue> {
        let mv = parse_movement(movement)?;
        Ok(self.inner.movements[mv.idx()].conflicting_flow)
    }

    /// Potential capacity c_p,x of a movement, veh/h (Equation 20-18).
    pub fn get_potential_capacity(&self, movement: &str) -> Result<Option<f64>, JsValue> {
        let mv = parse_movement(movement)?;
        Ok(self.inner.movements[mv.idx()].potential_capacity)
    }

    /// Movement capacity c_m,x, veh/h (Steps 6-9).
    pub fn get_movement_capacity(&self, movement: &str) -> Result<Option<f64>, JsValue> {
        let mv = parse_movement(movement)?;
        Ok(self.inner.movements[mv.idx()].movement_capacity)
    }

    /// Control delay of an exclusive-lane movement, s/veh (Equation 20-61).
    pub fn get_movement_delay(&self, movement: &str) -> Result<Option<f64>, JsValue> {
        let mv = parse_movement(movement)?;
        Ok(self.inner.movements[mv.idx()].control_delay)
    }

    /// LOS letter of an exclusive-lane movement (Exhibit 20-2).
    pub fn get_movement_los(&self, movement: &str) -> Result<Option<String>, JsValue> {
        let mv = parse_movement(movement)?;
        Ok(self.inner.movements[mv.idx()].los.map(|c| c.to_string()))
    }

    /// 95th percentile queue of a movement, veh (Equation 20-66).
    pub fn get_movement_queue_95(&self, movement: &str) -> Result<Option<f64>, JsValue> {
        let mv = parse_movement(movement)?;
        Ok(self.inner.movements[mv.idx()].queue_95)
    }

    /// Number of minor-street approach lanes ("NB" or "SB").
    pub fn get_lane_count(&self, approach: &str) -> Result<u32, JsValue> {
        Ok(self.lanes(approach)?.len() as u32)
    }

    /// Minor-approach lane result as an object with movements, flow_rate,
    /// capacity, control_delay, los, and queue_95.
    pub fn lane_result_to_js_value(&self, approach: &str, lane: usize) -> Result<JsValue, JsValue> {
        let lanes = self.lanes(approach)?;
        let l = lanes
            .get(lane)
            .ok_or_else(|| JsValue::from_str(&format!("no lane {lane} on {approach}")))?;
        Ok(lane_to_js_value(l))
    }

    /// Approach control delays [EB, WB, NB, SB], s/veh (Equation 20-64).
    /// Empty before `analyze()` has run.
    pub fn get_approach_delays(&self) -> Vec<f64> {
        self.inner
            .approach_delays
            .map(|d| d.to_vec())
            .unwrap_or_default()
    }

    /// Intersection control delay, s/veh (Equation 20-65). Note LOS is not
    /// defined for a TWSC intersection as a whole.
    pub fn get_intersection_delay(&self) -> Option<f64> {
        self.inner.intersection_delay
    }

    pub fn results_to_js_value(&self) -> JsValue {
        let obj = js_sys::Object::new();

        let movements = js_sys::Array::new();
        for &mv in ALL_MOVEMENTS.iter() {
            let m = &self.inner.movements[mv.idx()];
            let mo = js_sys::Object::new();
            set(&mo, "movement", JsValue::from_str(mv_label(mv)));
            set(&mo, "flow_rate", JsValue::from(m.flow_rate));
            set(&mo, "conflicting_flow", opt_num(m.conflicting_flow));
            set(&mo, "potential_capacity", opt_num(m.potential_capacity));
            set(&mo, "movement_capacity", opt_num(m.movement_capacity));
            set(&mo, "control_delay", opt_num(m.control_delay));
            set(&mo, "los", opt_char(m.los));
            set(&mo, "queue_95", opt_num(m.queue_95));
            movements.push(&JsValue::from(mo));
        }
        set(&obj, "movements", JsValue::from(movements));

        let lanes_nb = js_sys::Array::new();
        for l in &self.inner.lanes_nb {
            lanes_nb.push(&lane_to_js_value(l));
        }
        set(&obj, "lanes_nb", JsValue::from(lanes_nb));

        let lanes_sb = js_sys::Array::new();
        for l in &self.inner.lanes_sb {
            lanes_sb.push(&lane_to_js_value(l));
        }
        set(&obj, "lanes_sb", JsValue::from(lanes_sb));

        match self.inner.approach_delays {
            Some(d) => {
                let arr = js_sys::Array::new();
                for v in d.iter() {
                    arr.push(&JsValue::from(*v));
                }
                set(&obj, "approach_delays", JsValue::from(arr));
            }
            None => set(&obj, "approach_delays", JsValue::NULL),
        }
        set(&obj, "intersection_delay", opt_num(self.inner.intersection_delay));

        JsValue::from(obj)
    }
}

impl WasmTwsc {
    fn lanes(&self, approach: &str) -> Result<&Vec<TwscLaneResult>, JsValue> {
        match approach.to_ascii_uppercase().as_str() {
            "NB" => Ok(&self.inner.lanes_nb),
            "SB" => Ok(&self.inner.lanes_sb),
            other => Err(JsValue::from_str(&format!(
                "TWSC minor approach must be NB or SB, got {other}"
            ))),
        }
    }
}
