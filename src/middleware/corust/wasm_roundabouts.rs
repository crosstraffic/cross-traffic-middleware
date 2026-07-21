use wasm_bindgen::prelude::*;
use transportations_library::roundabouts::{
    BypassType, LaneAssignment, Leg, RoundaboutApproach, RoundaboutLaneResult, Roundabouts,
};

const LEGS: [(Leg, &str); 4] = [
    (Leg::NB, "nb"),
    (Leg::SB, "sb"),
    (Leg::EB, "eb"),
    (Leg::WB, "wb"),
];

fn parse_leg(leg: &str) -> Result<Leg, JsValue> {
    match leg.to_ascii_uppercase().as_str() {
        "NB" => Ok(Leg::NB),
        "SB" => Ok(Leg::SB),
        "EB" => Ok(Leg::EB),
        "WB" => Ok(Leg::WB),
        other => Err(JsValue::from_str(&format!(
            "entry must be NB/SB/EB/WB, got {other}"
        ))),
    }
}

fn parse_bypass(s: &str) -> Result<BypassType, JsValue> {
    match s.to_lowercase().as_str() {
        "none" => Ok(BypassType::None),
        "yielding" => Ok(BypassType::Yielding),
        "nonyielding" | "non_yielding" => Ok(BypassType::NonYielding),
        other => Err(JsValue::from_str(&format!(
            "bypass type must be none, yielding, or nonyielding, got {other}"
        ))),
    }
}

fn parse_lane_assignment(s: &str) -> Result<LaneAssignment, JsValue> {
    match s.to_lowercase().as_str() {
        "l_tr" => Ok(LaneAssignment::LeftAndThroughRight),
        "lt_r" => Ok(LaneAssignment::LeftThroughAndRight),
        "lt_tr" => Ok(LaneAssignment::LeftThroughAndThroughRight),
        "l_ltr" => Ok(LaneAssignment::LeftAndAllMovements),
        "ltr_r" => Ok(LaneAssignment::AllMovementsAndRight),
        other => Err(JsValue::from_str(&format!(
            "lane assignment must be l_tr, lt_r, lt_tr, l_ltr, or ltr_r, got {other}"
        ))),
    }
}

/// Build one entry from a flat numeric array laid out as
/// [v_u, v_l, v_t, v_r, heavy_vehicle_pct, entry_lanes, circulating_lanes,
/// exiting_lanes, n_ped]. Shorter arrays keep the defaults for the missing
/// tail (volumes 0, 0% heavy vehicles, one lane each, 0 p/h).
fn leg_from_flat(
    label: &str,
    vals: &[f64],
    bypass: Option<String>,
    lane_assignment: Option<String>,
) -> Result<RoundaboutApproach, JsValue> {
    if vals.len() > 9 {
        return Err(JsValue::from_str(&format!(
            "{label} entry array takes at most 9 values, got {}",
            vals.len()
        )));
    }
    let mut a = RoundaboutApproach::default();
    if let Some(&v) = vals.get(0) {
        a.v_u = v;
    }
    if let Some(&v) = vals.get(1) {
        a.v_l = v;
    }
    if let Some(&v) = vals.get(2) {
        a.v_t = v;
    }
    if let Some(&v) = vals.get(3) {
        a.v_r = v;
    }
    if let Some(&v) = vals.get(4) {
        a.heavy_vehicle_pct = v;
    }
    if let Some(&v) = vals.get(5) {
        a.entry_lanes = lane_value(label, "entry", v)?;
    }
    if let Some(&v) = vals.get(6) {
        a.circulating_lanes = lane_value(label, "circulating", v)?;
    }
    if let Some(&v) = vals.get(7) {
        a.exiting_lanes = lane_value(label, "exiting", v)?;
    }
    if let Some(&v) = vals.get(8) {
        a.n_ped = v;
    }
    if let Some(s) = bypass {
        a.bypass = parse_bypass(&s)?;
    }
    if let Some(s) = lane_assignment {
        a.lane_assignment = parse_lane_assignment(&s)?;
    }
    Ok(a)
}

fn lane_value(label: &str, kind: &str, v: f64) -> Result<u32, JsValue> {
    if v == 1.0 || v == 2.0 {
        Ok(v as u32)
    } else {
        Err(JsValue::from_str(&format!(
            "{label} {kind} lanes must be 1 or 2, got {v}"
        )))
    }
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

fn lane_to_js_value(l: &RoundaboutLaneResult) -> JsValue {
    let obj = js_sys::Object::new();
    set(&obj, "label", JsValue::from_str(&l.label));
    set(&obj, "flow_veh", JsValue::from(l.flow_veh));
    set(&obj, "capacity_veh", JsValue::from(l.capacity_veh));
    set(&obj, "v_c_ratio", JsValue::from(l.v_c_ratio));
    set(&obj, "control_delay", JsValue::from(l.control_delay));
    set(&obj, "los", JsValue::from_str(&l.los.to_string()));
    set(&obj, "queue_95", JsValue::from(l.queue_95));
    JsValue::from(obj)
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct WasmRoundabouts {
    inner: Roundabouts,
}

#[wasm_bindgen]
impl WasmRoundabouts {

    /// Each entry array is laid out as [v_u, v_l, v_t, v_r,
    /// heavy_vehicle_pct, entry_lanes, circulating_lanes, exiting_lanes,
    /// n_ped] with volumes in veh/h and pedestrians in p/h. Bypass types
    /// are "none", "yielding", or "nonyielding". Lane assignments apply to
    /// two-lane entries ("l_tr", "lt_r", "lt_tr", "l_ltr", "ltr_r").
    #[wasm_bindgen(constructor)]
    pub fn new(
        nb_entry: Vec<f64>,
        sb_entry: Vec<f64>,
        eb_entry: Vec<f64>,
        wb_entry: Vec<f64>,
        nb_bypass: Option<String>,
        sb_bypass: Option<String>,
        eb_bypass: Option<String>,
        wb_bypass: Option<String>,
        nb_lane_assignment: Option<String>,
        sb_lane_assignment: Option<String>,
        eb_lane_assignment: Option<String>,
        wb_lane_assignment: Option<String>,
        phf: Option<f64>,
        analysis_period_h: Option<f64>,
    ) -> Result<WasmRoundabouts, JsValue> {
        let nb = leg_from_flat("NB", &nb_entry, nb_bypass, nb_lane_assignment)?;
        let sb = leg_from_flat("SB", &sb_entry, sb_bypass, sb_lane_assignment)?;
        let eb = leg_from_flat("EB", &eb_entry, eb_bypass, eb_lane_assignment)?;
        let wb = leg_from_flat("WB", &wb_entry, wb_bypass, wb_lane_assignment)?;

        let mut inner = Roundabouts::new(nb, sb, eb, wb);
        if phf.is_some() {
            inner.phf = phf;
        }
        if let Some(v) = analysis_period_h {
            inner.analysis_period_h = v;
        }
        Ok(WasmRoundabouts { inner })
    }

    /// Run the complete HCM Chapter 22 procedure (Steps 1-12).
    pub fn analyze(&mut self) {
        self.inner.analyze();
    }

    /// Set a local calibration (A, B) for the entry capacity model
    /// (Equations 22-21 through 22-23; A = 3,600/t_f,
    /// B = (t_c - t_f/2)/3,600).
    pub fn set_calibration(&mut self, a: f64, b: f64) {
        self.inner.calibration = Some((a, b));
    }

    /// Conflicting circulating flow of an entry, pc/h (Equation 22-11).
    pub fn get_circulating_flow_pce(&self, entry: &str) -> Result<Option<f64>, JsValue> {
        Ok(self.approach(entry)?.circulating_flow_pce)
    }

    /// Number of entry lanes with results for an entry.
    pub fn get_lane_count(&self, entry: &str) -> Result<u32, JsValue> {
        Ok(self.approach(entry)?.lanes.len() as u32)
    }

    /// Entry-lane result (0 = left/only lane) as an object with label,
    /// flow_veh, capacity_veh, v_c_ratio, control_delay, los, and queue_95.
    pub fn lane_result_to_js_value(&self, entry: &str, lane: usize) -> Result<JsValue, JsValue> {
        let a = self.approach(entry)?;
        let l = a
            .lanes
            .get(lane)
            .ok_or_else(|| JsValue::from_str(&format!("no lane {lane} on {entry}")))?;
        Ok(lane_to_js_value(l))
    }

    /// Bypass-lane result object, or null if the entry has no bypass lane.
    pub fn bypass_result_to_js_value(&self, entry: &str) -> Result<JsValue, JsValue> {
        Ok(self
            .approach(entry)?
            .bypass_lane
            .as_ref()
            .map(lane_to_js_value)
            .unwrap_or(JsValue::NULL))
    }

    /// Approach control delay, s/veh (Equation 22-18, bypass included).
    pub fn get_approach_delay(&self, entry: &str) -> Result<Option<f64>, JsValue> {
        Ok(self.approach(entry)?.control_delay)
    }

    /// Approach LOS letter (Exhibit 22-8).
    pub fn get_approach_los(&self, entry: &str) -> Result<Option<String>, JsValue> {
        Ok(self.approach(entry)?.los.map(|c| c.to_string()))
    }

    /// Intersection control delay, s/veh (Equation 22-19).
    pub fn get_intersection_delay(&self) -> Option<f64> {
        self.inner.intersection_delay
    }

    /// Intersection LOS letter (Exhibit 22-8).
    pub fn get_intersection_los(&self) -> Option<String> {
        self.inner.intersection_los.map(|c| c.to_string())
    }

    pub fn results_to_js_value(&self) -> JsValue {
        let obj = js_sys::Object::new();
        for (leg, key) in LEGS.iter() {
            let a = self.inner.approach(*leg);
            let ao = js_sys::Object::new();
            set(&ao, "circulating_flow_pce", opt_num(a.circulating_flow_pce));
            let lanes = js_sys::Array::new();
            for l in &a.lanes {
                lanes.push(&lane_to_js_value(l));
            }
            set(&ao, "lanes", JsValue::from(lanes));
            set(
                &ao,
                "bypass",
                a.bypass_lane
                    .as_ref()
                    .map(lane_to_js_value)
                    .unwrap_or(JsValue::NULL),
            );
            set(&ao, "control_delay", opt_num(a.control_delay));
            set(&ao, "los", opt_char(a.los));
            set(&obj, key, JsValue::from(ao));
        }
        set(&obj, "intersection_delay", opt_num(self.inner.intersection_delay));
        set(&obj, "intersection_los", opt_char(self.inner.intersection_los));
        JsValue::from(obj)
    }
}

impl WasmRoundabouts {
    fn approach(&self, entry: &str) -> Result<&RoundaboutApproach, JsValue> {
        Ok(self.inner.approach(parse_leg(entry)?))
    }
}
