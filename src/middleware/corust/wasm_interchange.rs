use wasm_bindgen::prelude::*;
use transportations_library::ramp_terminals::Interchange;
use transportations_library::hcm::chapter23::alternative_intersections::{
    AltIntersectionForm, DisplacedLeftTurn, DltDelayCell,
};

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct WasmInterchange {
    inner: Interchange,
}

#[wasm_bindgen]
impl WasmInterchange {

    /// Build a signalized interchange ramp terminal analysis (HCM Ch.23
    /// Part B: diamond / parclo / SPUI / DDI) from a configuration object
    /// matching the serde schema of `hcm::chapter23::ramp_terminals::Interchange`
    /// (same shape as `tests/ExampleCases/hcm/RampTerminals/case1.json`):
    /// interchange form, cycle length, O-D demands A..N, and the per-lane-group
    /// geometry and signal timing.
    #[wasm_bindgen(constructor)]
    pub fn new(config: JsValue) -> Result<WasmInterchange, JsValue> {
        let inner: Interchange = serde_wasm_bindgen::from_value(config)
            .map_err(|e| JsValue::from_str(&format!("invalid interchange configuration: {e}")))?;
        Ok(WasmInterchange { inner })
    }

    /// Run the complete HCM Ch.23 Part B procedure (Steps 1-9 of Exhibit 23-22).
    pub fn analyze(&mut self) {
        self.inner.analyze();
    }

    pub fn get_cycle_length_s(&self) -> f64 {
        self.inner.get_cycle_length()
    }

    pub fn get_peak_hour_factor(&self) -> f64 {
        self.inner.get_peak_hour_factor()
    }

    /// Demand-weighted interchange experienced travel time ETT, s/veh
    /// (HCM Equation 23-52).
    pub fn get_interchange_ett_s(&self) -> Option<f64> {
        self.inner.get_interchange_ett()
    }

    /// Interchange LOS letter (HCM Exhibit 23-10), e.g. "C".
    pub fn get_interchange_los(&self) -> Option<String> {
        self.inner.get_interchange_los().map(|l| format!("{l:?}"))
    }

    /// O-D results as a JS array (movement letter, PHF-adjusted demand,
    /// control delay, EDTT, ETT, v/c and queue-storage flags, LOS).
    pub fn od_results_to_js_value(&self) -> JsValue {
        serde_wasm_bindgen::to_value(self.inner.get_od_results()).unwrap_or(JsValue::NULL)
    }

    /// Lane-group results as a JS array (movement, flow rate, saturation
    /// flow, effective green, capacity, v/c, delays, back of queue).
    pub fn lane_group_results_to_js_value(&self) -> JsValue {
        serde_wasm_bindgen::to_value(self.inner.get_results()).unwrap_or(JsValue::NULL)
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct WasmDisplacedLeftTurn {
    inner: DisplacedLeftTurn,
}

#[wasm_bindgen]
impl WasmDisplacedLeftTurn {

    /// Build a displaced left-turn (DLT) intersection analysis (HCM Ch.23
    /// Part C, Equation 23-69) from the per-junction weighted control-delay
    /// table of Exhibit 34-145.
    ///
    /// * `junction_flows` — flow rate v_j through each component junction, veh/h.
    /// * `junction_delays` — control delay d_j experienced by that flow, s/veh
    ///   (same order and length as `junction_flows`).
    /// * `total_od_demand_veh_h` — O-D demand total Σ v_OD, veh/h.
    /// * `full_dlt` — true for a full DLT, false or absent for a partial DLT.
    #[wasm_bindgen(constructor)]
    pub fn new(
        junction_flows: Vec<f64>,
        junction_delays: Vec<f64>,
        total_od_demand_veh_h: f64,
        full_dlt: Option<bool>,
    ) -> Self {
        let cells = junction_flows
            .iter()
            .zip(junction_delays.iter())
            .map(|(&flow_veh_h, &control_delay_s)| DltDelayCell {
                flow_veh_h,
                control_delay_s,
            })
            .collect();
        let form = if full_dlt.unwrap_or(false) {
            AltIntersectionForm::DltFull
        } else {
            AltIntersectionForm::DltPartial
        };
        WasmDisplacedLeftTurn {
            inner: DisplacedLeftTurn {
                form,
                cells,
                total_od_demand_veh_h,
            },
        }
    }

    /// Weighted-average intersection ETT (= control delay), s/veh
    /// (HCM Equation 23-69).
    pub fn get_intersection_ett_s(&self) -> f64 {
        self.inner.intersection_ett()
    }

    /// Intersection LOS letter (Chapter 19 control-delay thresholds).
    pub fn get_los(&self) -> String {
        format!("{:?}", self.inner.los())
    }
}
