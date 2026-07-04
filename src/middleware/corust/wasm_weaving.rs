use wasm_bindgen::prelude::*;
use transportations_library::weaving::{FacilityType, TerrainType, WeavingSegment, WeavingType};

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct WasmWeavingSegment {
    inner: WeavingSegment,
}

#[wasm_bindgen]
impl WasmWeavingSegment {

    #[wasm_bindgen(constructor)]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        weaving_type: Option<String>,
        facility_type: Option<String>,
        length_short: Option<f64>,
        num_lanes: Option<u32>,
        num_weaving_lanes: Option<u32>,
        ffs: Option<f64>,
        v_ff: Option<f64>,
        v_fr: Option<f64>,
        v_rf: Option<f64>,
        v_rr: Option<f64>,
        phf: Option<f64>,
        heavy_vehicle_pct: Option<f64>,
        terrain: Option<String>,
        lc_rf: Option<u32>,
        lc_fr: Option<u32>,
        lc_rr: Option<u32>,
        interchange_density: Option<f64>,
        basic_freeway_capacity: Option<f64>,
        caf: Option<f64>,
        saf: Option<f64>,
    ) -> Self {
        let mut inner = WeavingSegment::new();

        if let Some(wt) = weaving_type {
            inner.weaving_type = match wt.to_lowercase().as_str() {
                "two_sided" | "twosided" | "two-sided" => WeavingType::TwoSided,
                _ => WeavingType::OneSided,
            };
        }
        if let Some(ft) = facility_type {
            inner.facility_type = match ft.to_lowercase().as_str() {
                "multilane" | "cd" | "multilane_or_cd" => FacilityType::MultilaneOrCD,
                _ => FacilityType::Freeway,
            };
        }
        if let Some(t) = terrain {
            inner.terrain = match t.to_lowercase().as_str() {
                "rolling" => TerrainType::Rolling,
                "mountainous" => TerrainType::Mountainous,
                _ => TerrainType::Level,
            };
        }
        if let Some(v) = length_short {
            inner.length_short = v;
        }
        if let Some(v) = num_lanes {
            inner.num_lanes = v;
        }
        if let Some(v) = num_weaving_lanes {
            inner.num_weaving_lanes = v;
        }
        if let Some(v) = ffs {
            inner.ffs = v;
        }
        if let Some(v) = v_ff {
            inner.v_ff = v;
        }
        if let Some(v) = v_fr {
            inner.v_fr = v;
        }
        if let Some(v) = v_rf {
            inner.v_rf = v;
        }
        if let Some(v) = v_rr {
            inner.v_rr = v;
        }
        if let Some(v) = phf {
            inner.phf = v;
        }
        if let Some(v) = heavy_vehicle_pct {
            inner.heavy_vehicle_pct = v;
        }
        if let Some(v) = lc_rf {
            inner.lc_rf = v;
        }
        if let Some(v) = lc_fr {
            inner.lc_fr = v;
        }
        if let Some(v) = lc_rr {
            inner.lc_rr = v;
        }
        if let Some(v) = interchange_density {
            inner.interchange_density = v;
        }
        if let Some(v) = basic_freeway_capacity {
            inner.basic_freeway_capacity = v;
        }
        if let Some(v) = caf {
            inner.caf = v;
        }
        if let Some(v) = saf {
            inner.saf = v;
        }

        WasmWeavingSegment { inner }
    }

    /// Run the full HCM Ch.13 analysis (Steps 2-8) and return the LOS letter.
    /// Populates flows, capacity, lane-changing rates, speeds, and density.
    pub fn run_analysis(&mut self) -> String {
        let los: char = self.inner.run_analysis().into();
        los.to_string()
    }

    /// Step 2: demand flows under equivalent ideal conditions (Eq. 13-1).
    /// Returns [v_W, v_NW, v] in pc/h.
    pub fn determine_demand_flow(&mut self) -> Vec<f64> {
        let (v_w, v_nw, v) = self.inner.determine_demand_flow();
        vec![v_w, v_nw, v]
    }

    /// Step 3: minimum lane-changing rate LC_MIN (lc/h) - Eqs. 13-2/13-3.
    pub fn determine_configuration_characteristics(&mut self) -> f64 {
        self.inner.determine_configuration_characteristics()
    }

    /// Step 4: maximum weaving length L_MAX (ft) - Eq. 13-4.
    pub fn determine_max_weaving_length(&mut self) -> f64 {
        self.inner.determine_max_weaving_length()
    }

    /// Step 5: weaving segment capacity (veh/h) - Eqs. 13-5..13-10.
    pub fn determine_capacity(&mut self) -> f64 {
        self.inner.determine_capacity()
    }

    /// Step 6: total lane-changing rate LC_ALL (lc/h) - Eqs. 13-11..13-17.
    pub fn determine_lane_changing_rates(&mut self) -> f64 {
        self.inner.determine_lane_changing_rates()
    }

    /// Step 7: speeds [S_W, S_NW, S] in mi/h - Eqs. 13-18..13-22.
    pub fn estimate_speed(&mut self) -> Vec<f64> {
        let (s_w, s_nw, s) = self.inner.estimate_speed();
        vec![s_w, s_nw, s]
    }

    /// Step 8a: density (pc/mi/ln) - Eq. 13-23.
    pub fn determine_density(&mut self) -> f64 {
        self.inner.determine_density()
    }

    /// Step 8b: level of service letter - Exhibit 13-6.
    pub fn determine_los(&mut self) -> String {
        let los: char = self.inner.determine_los().into();
        los.to_string()
    }

    pub fn get_flow_weaving(&self) -> f64 {
        self.inner.get_flow_weaving()
    }

    pub fn get_flow_nonweaving(&self) -> f64 {
        self.inner.get_flow_nonweaving()
    }

    pub fn get_flow_total(&self) -> f64 {
        self.inner.get_flow_total()
    }

    pub fn get_volume_ratio(&self) -> f64 {
        self.inner.get_volume_ratio()
    }

    pub fn get_lc_min(&self) -> f64 {
        self.inner.get_lc_min()
    }

    pub fn get_l_max(&self) -> f64 {
        self.inner.get_l_max()
    }

    pub fn is_weaving(&self) -> bool {
        self.inner.is_weaving_segment()
    }

    pub fn get_capacity(&self) -> f64 {
        self.inner.get_capacity()
    }

    pub fn get_vc_ratio(&self) -> f64 {
        self.inner.get_vc_ratio()
    }

    pub fn get_lc_all(&self) -> f64 {
        self.inner.get_lc_all()
    }

    pub fn get_speed_weaving(&self) -> f64 {
        self.inner.get_speed_weaving()
    }

    pub fn get_speed_nonweaving(&self) -> f64 {
        self.inner.get_speed_nonweaving()
    }

    pub fn get_speed_avg(&self) -> f64 {
        self.inner.get_speed_avg()
    }

    pub fn get_density(&self) -> f64 {
        self.inner.get_density()
    }

    pub fn get_los(&self) -> Option<String> {
        self.inner.get_los().map(|l| {
            let c: char = l.into();
            c.to_string()
        })
    }

    pub fn results_to_js_value(&self) -> JsValue {
        let obj = js_sys::Object::new();
        js_sys::Reflect::set(&obj, &JsValue::from_str("flow_weaving"), &JsValue::from(self.get_flow_weaving())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("flow_nonweaving"), &JsValue::from(self.get_flow_nonweaving())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("flow_total"), &JsValue::from(self.get_flow_total())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("volume_ratio"), &JsValue::from(self.get_volume_ratio())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("lc_min"), &JsValue::from(self.get_lc_min())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("l_max"), &JsValue::from(self.get_l_max())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("is_weaving"), &JsValue::from(self.is_weaving())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("capacity"), &JsValue::from(self.get_capacity())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("vc_ratio"), &JsValue::from(self.get_vc_ratio())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("lc_all"), &JsValue::from(self.get_lc_all())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("speed_weaving"), &JsValue::from(self.get_speed_weaving())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("speed_nonweaving"), &JsValue::from(self.get_speed_nonweaving())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("speed_avg"), &JsValue::from(self.get_speed_avg())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("density"), &JsValue::from(self.get_density())).unwrap();

        JsValue::from(obj)
    }
}
