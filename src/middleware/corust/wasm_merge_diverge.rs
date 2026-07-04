use wasm_bindgen::prelude::*;
use transportations_library::merge_diverge::{
    AdjacentRampType, RampLanes, RampSegment, RampSide, RampType, TerrainType,
};

fn parse_adjacent(s: &str) -> AdjacentRampType {
    match s.to_lowercase().as_str() {
        "on_ramp" | "onramp" | "on-ramp" | "on" => AdjacentRampType::OnRamp,
        "off_ramp" | "offramp" | "off-ramp" | "off" => AdjacentRampType::OffRamp,
        _ => AdjacentRampType::None,
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct WasmRampSegment {
    inner: RampSegment,
}

#[wasm_bindgen]
impl WasmRampSegment {

    #[wasm_bindgen(constructor)]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        ramp_type: Option<String>,
        ramp_side: Option<String>,
        ramp_lanes: Option<u32>,
        freeway_lanes: Option<u32>,
        freeway_ffs: Option<f64>,
        ramp_ffs: Option<f64>,
        accel_lane_length: Option<f64>,
        accel_lane_length2: Option<f64>,
        decel_lane_length: Option<f64>,
        decel_lane_length2: Option<f64>,
        freeway_demand: Option<f64>,
        ramp_demand: Option<f64>,
        phf: Option<f64>,
        heavy_vehicle_pct: Option<f64>,
        ramp_heavy_vehicle_pct: Option<f64>,
        terrain: Option<String>,
        adjacent_upstream: Option<String>,
        upstream_distance: Option<f64>,
        upstream_ramp_flow: Option<f64>,
        adjacent_downstream: Option<String>,
        downstream_distance: Option<f64>,
        downstream_ramp_flow: Option<f64>,
        caf: Option<f64>,
        saf: Option<f64>,
    ) -> Self {
        let mut inner = RampSegment::new();

        if let Some(rt) = ramp_type {
            inner.ramp_type = match rt.to_lowercase().as_str() {
                "off_ramp" | "offramp" | "off-ramp" | "diverge" => RampType::OffRamp,
                "major_merge" => RampType::MajorMerge,
                "major_diverge" => RampType::MajorDiverge,
                _ => RampType::OnRamp,
            };
        }
        if let Some(rs) = ramp_side {
            inner.ramp_side = match rs.to_lowercase().as_str() {
                "left" => RampSide::Left,
                _ => RampSide::Right,
            };
        }
        if let Some(rl) = ramp_lanes {
            inner.ramp_lanes = match rl {
                2 => RampLanes::TwoLane,
                _ => RampLanes::OneLane,
            };
        }
        if let Some(t) = terrain {
            inner.terrain = match t.to_lowercase().as_str() {
                "rolling" => TerrainType::Rolling,
                "mountainous" => TerrainType::Mountainous,
                _ => TerrainType::Level,
            };
        }
        if let Some(a) = adjacent_upstream {
            inner.adjacent_upstream = parse_adjacent(&a);
        }
        if let Some(a) = adjacent_downstream {
            inner.adjacent_downstream = parse_adjacent(&a);
        }
        if let Some(v) = freeway_lanes {
            inner.freeway_lanes = v;
        }
        if let Some(v) = freeway_ffs {
            inner.freeway_ffs = v;
        }
        if let Some(v) = ramp_ffs {
            inner.ramp_ffs = v;
        }
        if accel_lane_length.is_some() {
            inner.accel_lane_length = accel_lane_length;
        }
        if accel_lane_length2.is_some() {
            inner.accel_lane_length2 = accel_lane_length2;
        }
        if decel_lane_length.is_some() {
            inner.decel_lane_length = decel_lane_length;
        }
        if decel_lane_length2.is_some() {
            inner.decel_lane_length2 = decel_lane_length2;
        }
        if let Some(v) = freeway_demand {
            inner.freeway_demand = v;
        }
        if let Some(v) = ramp_demand {
            inner.ramp_demand = v;
        }
        if let Some(v) = phf {
            inner.phf = v;
        }
        if let Some(v) = heavy_vehicle_pct {
            inner.heavy_vehicle_pct = v;
        }
        if ramp_heavy_vehicle_pct.is_some() {
            inner.ramp_heavy_vehicle_pct = ramp_heavy_vehicle_pct;
        }
        if upstream_distance.is_some() {
            inner.upstream_distance = upstream_distance;
        }
        if upstream_ramp_flow.is_some() {
            inner.upstream_ramp_flow = upstream_ramp_flow;
        }
        if downstream_distance.is_some() {
            inner.downstream_distance = downstream_distance;
        }
        if downstream_ramp_flow.is_some() {
            inner.downstream_ramp_flow = downstream_ramp_flow;
        }
        if let Some(v) = caf {
            inner.caf = v;
        }
        if let Some(v) = saf {
            inner.saf = v;
        }

        WasmRampSegment { inner }
    }

    /// Run the full HCM Ch.14 analysis (Steps 1-5) and return the LOS letter.
    /// Populates flows, v_12, capacities, density, and speeds.
    pub fn run_analysis(&mut self) -> String {
        let los: char = self.inner.run_analysis().into();
        los.to_string()
    }

    /// Step 1: demand flows [v_F, v_R] in pc/h - Eq. 14-1.
    pub fn determine_demand_flow(&mut self) -> Vec<f64> {
        let (v_f, v_r) = self.inner.determine_demand_flow();
        vec![v_f, v_r]
    }

    /// Step 2: flow in Lanes 1 and 2, v_12 (pc/h) - Eqs. 14-2..14-19.
    pub fn estimate_v12(&mut self) -> f64 {
        self.inner.estimate_v12()
    }

    /// Step 3: adjusted freeway capacity (pc/h) and capacity checks
    /// (Exhibits 14-10/14-12, Eq. 14-21).
    pub fn determine_capacity(&mut self) -> f64 {
        self.inner.determine_capacity()
    }

    /// Step 4: density in the ramp influence area (pc/mi/ln)
    /// - Eqs. 14-22/14-23/14-28.
    pub fn determine_density(&mut self) -> f64 {
        self.inner.determine_density()
    }

    /// Level of service letter - Exhibit 14-3.
    pub fn determine_los(&mut self) -> String {
        let los: char = self.inner.determine_los().into();
        los.to_string()
    }

    /// Step 5: speeds [S_R, S_O, S] in mi/h - Exhibits 14-13/14-14/14-15.
    /// S_O is NaN when the outer-lane speed does not apply.
    pub fn estimate_speed(&mut self) -> Vec<f64> {
        let (s_r, s_o, s) = self.inner.estimate_speed();
        vec![s_r, s_o.unwrap_or(f64::NAN), s]
    }

    pub fn get_flow_freeway(&self) -> f64 {
        self.inner.get_flow_freeway()
    }

    pub fn get_flow_ramp(&self) -> f64 {
        self.inner.get_flow_ramp()
    }

    pub fn get_p_f(&self) -> Option<f64> {
        self.inner.p_f
    }

    pub fn get_v12(&self) -> f64 {
        self.inner.get_v12()
    }

    pub fn get_vr12(&self) -> f64 {
        self.inner.get_vr12()
    }

    pub fn get_capacity_freeway(&self) -> f64 {
        self.inner.get_capacity_freeway()
    }

    pub fn get_capacity_ramp(&self) -> f64 {
        self.inner.get_capacity_ramp()
    }

    pub fn get_vc_ratio(&self) -> f64 {
        self.inner.get_vc_ratio()
    }

    pub fn get_demand_exceeds_capacity(&self) -> Option<bool> {
        self.inner.demand_exceeds_capacity
    }

    pub fn get_exceeds_max_desirable(&self) -> Option<bool> {
        self.inner.exceeds_max_desirable
    }

    pub fn get_density(&self) -> f64 {
        self.inner.get_density()
    }

    pub fn get_speed_ramp(&self) -> f64 {
        self.inner.get_speed_ramp()
    }

    pub fn get_speed_outer(&self) -> Option<f64> {
        self.inner.get_speed_outer()
    }

    pub fn get_speed_avg(&self) -> f64 {
        self.inner.get_speed_avg()
    }

    pub fn get_los(&self) -> Option<String> {
        self.inner.get_los().map(|l| {
            let c: char = l.into();
            c.to_string()
        })
    }

    pub fn results_to_js_value(&self) -> JsValue {
        let obj = js_sys::Object::new();
        js_sys::Reflect::set(&obj, &JsValue::from_str("flow_freeway"), &JsValue::from(self.get_flow_freeway())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("flow_ramp"), &JsValue::from(self.get_flow_ramp())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("v12"), &JsValue::from(self.get_v12())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("vr12"), &JsValue::from(self.get_vr12())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("capacity_freeway"), &JsValue::from(self.get_capacity_freeway())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("capacity_ramp"), &JsValue::from(self.get_capacity_ramp())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("vc_ratio"), &JsValue::from(self.get_vc_ratio())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("density"), &JsValue::from(self.get_density())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("speed_ramp"), &JsValue::from(self.get_speed_ramp())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("speed_avg"), &JsValue::from(self.get_speed_avg())).unwrap();

        JsValue::from(obj)
    }
}
