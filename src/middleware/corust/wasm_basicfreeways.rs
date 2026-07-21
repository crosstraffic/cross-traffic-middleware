use wasm_bindgen::prelude::*;
use transportations_library::basicfreeways::BasicFreeways;
use transportations_library::hcm::common::CityType;

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct WasmBasicFreeways {
    inner: BasicFreeways,
}

#[wasm_bindgen]
impl WasmBasicFreeways {

    #[wasm_bindgen(constructor)]
    pub fn new(
        bffs: Option<f64>,
        lane_width: Option<f64>,
        lane_count: Option<u32>,
        lc_r: Option<u32>,
        lc_l: Option<u32>,
        trd: Option<u32>,
        apd: Option<u32>,
        grade: Option<f64>,
        terrain_type: Option<String>,
        speed_limit: Option<u32>,
        phf: Option<f64>,
        p_t: Option<f64>,
        demand_flow_i: Option<f64>,
        length: Option<f64>,
        highway_type: Option<String>,
        city_type: Option<String>,
    ) -> Self {
        let mut inner = BasicFreeways::new();
        if let Some(v) = bffs {
            inner.bffs = v;
        }
        if lane_width.is_some() {
            inner.lw = lane_width;
        }
        if let Some(v) = lane_count {
            inner.lane_count = v;
        }
        if let Some(v) = lc_r {
            inner.lc_r = v;
        }
        if let Some(v) = lc_l {
            inner.lc_l = v;
        }
        if let Some(v) = trd {
            inner.trd = v;
        }
        if let Some(v) = apd {
            inner.apd = v;
        }
        if let Some(v) = grade {
            inner.grade = v;
        }
        if terrain_type.is_some() {
            inner.terrain_type = terrain_type;
        }
        if let Some(v) = speed_limit {
            inner.speed_limit = v;
        }
        if let Some(v) = phf {
            inner.phf = v;
        }
        if p_t.is_some() {
            inner.p_t = p_t;
        }
        if let Some(v) = demand_flow_i {
            inner.demand_flow_i = v;
        }
        if let Some(v) = length {
            inner.length = v;
        }
        if let Some(v) = highway_type {
            inner.highway_type = v;
        }
        if let Some(ct) = city_type {
            inner.city_type = match ct.to_lowercase().as_str() {
                "rural" => CityType::Rural,
                _ => CityType::Urban,
            };
        }
        WasmBasicFreeways { inner }
    }

    /// Run the full HCM Ch.12 operational analysis and return the LOS letter.
    /// Populates ffs, capacity, speed, density, and v/c ratio.
    pub fn run_operational_analysis(&mut self) -> Result<String, JsValue> {
        let los = self.inner
            .run_operational_analysis()
            .map_err(|e| JsValue::from_str(&e))?;
        let los: char = los.into();
        Ok(los.to_string())
    }

    pub fn determine_free_flow_speed(&mut self) -> f64 {
        self.inner.determine_free_flow_speed()
    }

    pub fn get_ffs(&self) -> f64 {
        self.inner.ffs
    }

    pub fn get_capacity(&self) -> f64 {
        self.inner.capacity
    }

    pub fn get_adjusted_capacity(&self) -> f64 {
        self.inner.capacity_adj
    }

    pub fn get_speed(&self) -> f64 {
        self.inner.speed
    }

    pub fn get_density(&self) -> f64 {
        self.inner.density
    }

    pub fn get_vc_ratio(&self) -> f64 {
        self.inner.vc_ratio
    }

    pub fn get_lane_count(&self) -> u32 {
        self.inner.lane_count
    }

    pub fn results_to_js_value(&self) -> JsValue {
        let obj = js_sys::Object::new();
        js_sys::Reflect::set(&obj, &JsValue::from_str("ffs"), &JsValue::from(self.get_ffs())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("capacity"), &JsValue::from(self.get_capacity())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("adjusted_capacity"), &JsValue::from(self.get_adjusted_capacity())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("speed"), &JsValue::from(self.get_speed())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("density"), &JsValue::from(self.get_density())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("vc_ratio"), &JsValue::from(self.get_vc_ratio())).unwrap();

        JsValue::from(obj)
    }
}
