use wasm_bindgen::prelude::*;
use transportations_library::managed_lanes::{ManagedLaneSegment, ManagedLaneType};

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct WasmManagedLanes {
    inner: ManagedLaneSegment,
}

#[wasm_bindgen]
impl WasmManagedLanes {

    #[wasm_bindgen(constructor)]
    pub fn new(
        lane_type: Option<String>,
        ffs: Option<f64>,
        demand: Option<f64>,
        gp_density: Option<f64>,
        caf: Option<f64>,
        saf: Option<f64>,
    ) -> Self {
        let lt = match lane_type.as_deref().map(str::to_lowercase).as_deref() {
            Some("buffer1") => ManagedLaneType::Buffer1,
            Some("buffer2") => ManagedLaneType::Buffer2,
            Some("barrier1") => ManagedLaneType::Barrier1,
            Some("barrier2") => ManagedLaneType::Barrier2,
            _ => ManagedLaneType::ContinuousAccess,
        };
        let mut inner = ManagedLaneSegment::new(lt, ffs.unwrap_or(65.0));
        if let Some(v) = demand {
            inner.set_demand(v);
        }
        if let Some(v) = gp_density {
            inner.set_gp_density(v);
        }
        if let Some(v) = caf {
            inner.set_caf(v);
        }
        if let Some(v) = saf {
            inner.set_saf(v);
        }
        WasmManagedLanes { inner }
    }

    /// Run the full HCM Ch.12 Section 4 managed lane analysis and return the LOS letter.
    /// Populates breakpoint, adjusted capacity, speed, and density.
    pub fn run_analysis(&mut self) -> String {
        let los: char = self.inner.run_analysis().into();
        los.to_string()
    }

    /// Breakpoint BP (pc/h/ln) - Equation 12-13.
    pub fn calculate_breakpoint(&mut self) -> f64 {
        self.inner.calculate_ffs_adj();
        self.inner.calculate_breakpoint()
    }

    /// Adjusted capacity c_adj (pc/h/ln) - Equation 12-14.
    pub fn calculate_capacity(&mut self) -> f64 {
        self.inner.calculate_ffs_adj();
        self.inner.calculate_capacity()
    }

    /// Space mean speed S_ML (mi/h) - Equation 12-12.
    pub fn calculate_speed(&mut self) -> f64 {
        self.inner.calculate_speed()
    }

    /// Density (pc/mi/ln).
    pub fn calculate_density(&mut self) -> f64 {
        self.inner.calculate_density()
    }

    /// Level of service letter (Exhibit 12-15 criteria).
    pub fn determine_los(&mut self) -> String {
        let los: char = self.inner.determine_los().into();
        los.to_string()
    }

    pub fn set_demand(&mut self, v_p: f64) {
        self.inner.set_demand(v_p);
    }

    pub fn set_gp_density(&mut self, k_gp: f64) {
        self.inner.set_gp_density(k_gp);
    }

    pub fn get_breakpoint(&self) -> f64 {
        self.inner.breakpoint
    }

    pub fn get_capacity(&self) -> f64 {
        self.inner.capacity_adj
    }

    pub fn get_speed(&self) -> f64 {
        self.inner.speed
    }

    pub fn get_density(&self) -> f64 {
        self.inner.density
    }

    pub fn get_los(&self) -> Option<String> {
        self.inner.los.map(|l| {
            let c: char = l.into();
            c.to_string()
        })
    }

    /// Whether the segment type is subject to GP-lane friction
    /// (continuous access and Buffer 1 types).
    pub fn has_friction_effect(&self) -> bool {
        self.inner.has_friction_effect()
    }

    /// Whether friction is active (K_GP > 35 pc/mi/ln on a friction type).
    pub fn is_friction_active(&self) -> bool {
        self.inner.is_friction_active()
    }

    pub fn results_to_js_value(&self) -> JsValue {
        let obj = js_sys::Object::new();
        js_sys::Reflect::set(&obj, &JsValue::from_str("breakpoint"), &JsValue::from(self.get_breakpoint())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("capacity"), &JsValue::from(self.get_capacity())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("speed"), &JsValue::from(self.get_speed())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("density"), &JsValue::from(self.get_density())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("friction_active"), &JsValue::from(self.is_friction_active())).unwrap();

        JsValue::from(obj)
    }
}
