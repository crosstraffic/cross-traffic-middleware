use wasm_bindgen::prelude::*;
use transportations_library::awsc::{ApproachDir, Awsc, AwscApproach, AwscLane};

const DIRS: [(ApproachDir, &str); 4] = [
    (ApproachDir::EB, "eb"),
    (ApproachDir::WB, "wb"),
    (ApproachDir::NB, "nb"),
    (ApproachDir::SB, "sb"),
];

fn parse_dir(dir: &str) -> Result<ApproachDir, JsValue> {
    match dir.to_ascii_uppercase().as_str() {
        "EB" => Ok(ApproachDir::EB),
        "WB" => Ok(ApproachDir::WB),
        "NB" => Ok(ApproachDir::NB),
        "SB" => Ok(ApproachDir::SB),
        other => Err(JsValue::from_str(&format!(
            "approach must be EB/WB/NB/SB, got {other}"
        ))),
    }
}

/// Build one approach from a flat [left, through, right, left, through,
/// right, ...] volume array (one triple per lane, left-to-right; an empty
/// array marks a leg with no approach, e.g., the fourth leg of a T).
fn approach_from_flat(
    label: &str,
    volumes: &[f64],
    heavy_vehicle_pct: Option<f64>,
) -> Result<AwscApproach, JsValue> {
    if volumes.len() % 3 != 0 {
        return Err(JsValue::from_str(&format!(
            "{label} lane volumes must come in [left, through, right] triples, got {} values",
            volumes.len()
        )));
    }
    if volumes.len() > 9 {
        return Err(JsValue::from_str(&format!(
            "{label} supports at most 3 lanes, got {}",
            volumes.len() / 3
        )));
    }
    let lanes: Vec<AwscLane> = volumes
        .chunks(3)
        .map(|c| AwscLane::new(c[0], c[1], c[2]))
        .collect();
    Ok(AwscApproach {
        lanes,
        heavy_vehicle_pct: heavy_vehicle_pct.unwrap_or(0.0),
        ..Default::default()
    })
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

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct WasmAwsc {
    inner: Awsc,
}

#[wasm_bindgen]
impl WasmAwsc {

    /// Lane volumes are flat arrays of [left, through, right] triples in
    /// veh/h, one triple per lane (left-to-right). Pass an empty array for
    /// a leg with no approach (three-leg intersection).
    #[wasm_bindgen(constructor)]
    pub fn new(
        eb_lane_volumes: Vec<f64>,
        wb_lane_volumes: Vec<f64>,
        nb_lane_volumes: Vec<f64>,
        sb_lane_volumes: Vec<f64>,
        eb_heavy_vehicle_pct: Option<f64>,
        wb_heavy_vehicle_pct: Option<f64>,
        nb_heavy_vehicle_pct: Option<f64>,
        sb_heavy_vehicle_pct: Option<f64>,
        phf: Option<f64>,
        analysis_period_h: Option<f64>,
    ) -> Result<WasmAwsc, JsValue> {
        let eb = approach_from_flat("EB", &eb_lane_volumes, eb_heavy_vehicle_pct)?;
        let wb = approach_from_flat("WB", &wb_lane_volumes, wb_heavy_vehicle_pct)?;
        let nb = approach_from_flat("NB", &nb_lane_volumes, nb_heavy_vehicle_pct)?;
        let sb = approach_from_flat("SB", &sb_lane_volumes, sb_heavy_vehicle_pct)?;

        let mut inner = Awsc::new(eb, wb, nb, sb);
        if phf.is_some() {
            inner.phf = phf;
        }
        if let Some(v) = analysis_period_h {
            inner.analysis_period_h = v;
        }
        Ok(WasmAwsc { inner })
    }

    /// Run the HCM Chapter 21 procedure (Steps 1-16 except the Step 12
    /// capacity search; see `compute_lane_capacity`).
    pub fn analyze(&mut self) {
        self.inner.analyze();
    }

    /// Iterations used by the departure-headway convergence loop.
    pub fn get_iterations(&self) -> Option<u32> {
        self.inner.iterations
    }

    /// Number of lanes on an approach ("EB"/"WB"/"NB"/"SB").
    pub fn get_lane_count(&self, approach: &str) -> Result<u32, JsValue> {
        Ok(self.inner.approach(parse_dir(approach)?).lanes.len() as u32)
    }

    /// Converged departure headway h_d of a lane, s (Equation 21-28).
    pub fn get_departure_headway(&self, approach: &str, lane: usize) -> Result<Option<f64>, JsValue> {
        Ok(self.lane(approach, lane)?.departure_headway)
    }

    /// Degree of utilization x = v h_d / 3,600 (Equation 21-14).
    pub fn get_degree_of_utilization(&self, approach: &str, lane: usize) -> Result<Option<f64>, JsValue> {
        Ok(self.lane(approach, lane)?.degree_of_utilization)
    }

    /// Service time t_s = h_d - m, s (Equation 21-29).
    pub fn get_service_time(&self, approach: &str, lane: usize) -> Result<Option<f64>, JsValue> {
        Ok(self.lane(approach, lane)?.service_time)
    }

    /// Lane control delay, s/veh (Equation 21-30).
    pub fn get_lane_delay(&self, approach: &str, lane: usize) -> Result<Option<f64>, JsValue> {
        Ok(self.lane(approach, lane)?.control_delay)
    }

    /// Lane LOS letter (Exhibit 21-8).
    pub fn get_lane_los(&self, approach: &str, lane: usize) -> Result<Option<String>, JsValue> {
        Ok(self.lane(approach, lane)?.los.map(|c| c.to_string()))
    }

    /// Lane 95th percentile queue, veh (Equation 21-33).
    pub fn get_lane_queue_95(&self, approach: &str, lane: usize) -> Result<Option<f64>, JsValue> {
        Ok(self.lane(approach, lane)?.queue_95)
    }

    /// Step 12 capacity of a lane, veh/h (iterative search; expensive).
    pub fn compute_lane_capacity(&mut self, approach: &str, lane: usize) -> Result<f64, JsValue> {
        let dir = parse_dir(approach)?;
        if lane >= self.inner.approach(dir).lanes.len() {
            return Err(JsValue::from_str(&format!("no lane {lane} on {approach}")));
        }
        // Ensure flows/groups/adjustments are populated
        self.inner.step1_2_flow_rates();
        self.inner.step3_geometry_groups();
        self.inner.step4_headway_adjustments();
        Ok(self.inner.capacity_of_lane(dir, lane))
    }

    /// Approach control delay, s/veh (Equation 21-31).
    pub fn get_approach_delay(&self, approach: &str) -> Result<Option<f64>, JsValue> {
        Ok(self.inner.approach(parse_dir(approach)?).control_delay)
    }

    /// Approach LOS letter (Exhibit 21-8).
    pub fn get_approach_los(&self, approach: &str) -> Result<Option<String>, JsValue> {
        Ok(self
            .inner
            .approach(parse_dir(approach)?)
            .los
            .map(|c| c.to_string()))
    }

    /// Intersection control delay, s/veh (Equation 21-32).
    pub fn get_intersection_delay(&self) -> Option<f64> {
        self.inner.intersection_delay
    }

    /// Intersection LOS letter (Exhibit 21-8).
    pub fn get_intersection_los(&self) -> Option<String> {
        self.inner.intersection_los.map(|c| c.to_string())
    }

    pub fn results_to_js_value(&self) -> JsValue {
        let obj = js_sys::Object::new();
        for (dir, key) in DIRS.iter() {
            let a = self.inner.approach(*dir);
            let ao = js_sys::Object::new();
            let lanes = js_sys::Array::new();
            for l in &a.lanes {
                let lo = js_sys::Object::new();
                set(&lo, "flow_rate", opt_num(l.flow_rate));
                set(&lo, "departure_headway", opt_num(l.departure_headway));
                set(&lo, "degree_of_utilization", opt_num(l.degree_of_utilization));
                set(&lo, "service_time", opt_num(l.service_time));
                set(&lo, "control_delay", opt_num(l.control_delay));
                set(&lo, "los", opt_char(l.los));
                set(&lo, "queue_95", opt_num(l.queue_95));
                lanes.push(&JsValue::from(lo));
            }
            set(&ao, "lanes", JsValue::from(lanes));
            set(&ao, "control_delay", opt_num(a.control_delay));
            set(&ao, "los", opt_char(a.los));
            set(&obj, key, JsValue::from(ao));
        }
        set(&obj, "intersection_delay", opt_num(self.inner.intersection_delay));
        set(&obj, "intersection_los", opt_char(self.inner.intersection_los));
        match self.inner.iterations {
            Some(i) => set(&obj, "iterations", JsValue::from(i)),
            None => set(&obj, "iterations", JsValue::NULL),
        }
        JsValue::from(obj)
    }
}

impl WasmAwsc {
    fn lane(&self, approach: &str, lane: usize) -> Result<&AwscLane, JsValue> {
        let dir = parse_dir(approach)?;
        self.inner
            .approach(dir)
            .lanes
            .get(lane)
            .ok_or_else(|| JsValue::from_str(&format!("no lane {lane} on {approach}")))
    }
}
