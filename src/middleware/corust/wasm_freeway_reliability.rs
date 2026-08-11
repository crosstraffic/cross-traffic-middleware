use wasm_bindgen::prelude::*;
use transportations_library::hcm::chapter11::reliability::ReliabilityAnalysis;
use transportations_library::hcm::chapter11::scenario_generation::{
    FreewayScenario, IncidentInputs, Weekday,
};

use super::wasm_freeway_facilities::{build_facility, WasmFacilitySegment};

fn parse_weekday(s: &str) -> Weekday {
    match s.to_lowercase().as_str() {
        "tuesday" | "tue" => Weekday::Tuesday,
        "wednesday" | "wed" => Weekday::Wednesday,
        "thursday" | "thu" => Weekday::Thursday,
        "friday" | "fri" => Weekday::Friday,
        "saturday" | "sat" => Weekday::Saturday,
        "sunday" | "sun" => Weekday::Sunday,
        _ => Weekday::Monday,
    }
}

/// HCM Chapter 11 freeway reliability analysis (Steps B-1 through B-13),
/// scoped to demand variability plus optional incidents. The scenario
/// generator defaults to a whole-year reliability reporting period
/// (12 months, Monday through Friday, Exhibit 11-18 urban demand ratios).
/// Weather events, work zones, and special events are not exposed by this
/// binding.
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct WasmFreewayReliability {
    inner: ReliabilityAnalysis,
}

#[wasm_bindgen]
impl WasmFreewayReliability {

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
        months: Vec<u32>,
        replications: Option<u32>,
        seed_month: Option<u32>,
        seed_weekday: Option<String>,
        crash_rate_per_100mvmt: Option<f64>,
        incident_to_crash_ratio: Option<f64>,
        rng_seed: Option<u32>,
        vmt_weighted: Option<bool>,
    ) -> Self {
        let facility = build_facility(
            &wasm_segments,
            mainline_demand,
            ffs,
            heavy_vehicle_pct,
            terrain,
            city_type,
            phf,
            None,
            None,
            None,
            None,
        );

        let mut inner = ReliabilityAnalysis::default();
        inner.facility = facility;
        if !months.is_empty() {
            inner.scenario_generation.months = months;
        }
        if let Some(v) = replications {
            inner.scenario_generation.replications = v;
        }
        if let Some(v) = seed_month {
            inner.scenario_generation.seed_month = v;
        }
        if let Some(day) = seed_weekday {
            inner.scenario_generation.seed_weekday = parse_weekday(&day);
        }
        if let Some(rate) = crash_rate_per_100mvmt {
            let mut incidents = IncidentInputs::default();
            incidents.crash_rate_per_100mvmt = Some(rate);
            if let Some(ratio) = incident_to_crash_ratio {
                incidents.incident_to_crash_ratio = ratio;
            }
            inner.scenario_generation.incidents = Some(incidents);
        }
        if let Some(seed) = rng_seed {
            inner.scenario_generation.rng_seed = u64::from(seed);
        }
        if let Some(v) = vmt_weighted {
            inner.vmt_weighted = v;
        }
        WasmFreewayReliability { inner }
    }

    /// Run the full reliability methodology (scenario generation plus one
    /// Chapter 10 core-methodology evaluation per scenario). Throws with the
    /// validation message on invalid input.
    pub fn run(&mut self) -> Result<(), JsValue> {
        self.inner.run().map_err(|e| JsValue::from_str(&e))
    }

    /// Seed-file VMT over the whole study period, veh-mi (Equation 25-88).
    /// This is the denominator the incident frequencies of Equation 25-77
    /// are built on, so it is available before `run()` and does not depend
    /// on the Monte Carlo draw.
    pub fn seed_total_vmt(&self) -> f64 {
        self.inner.seed_statistics().total_vmt()
    }

    /// Number of 15-min analysis periods in the seed file, i.e. the study
    /// period length D_SP in quarter hours.
    pub fn seed_num_periods(&self) -> usize {
        self.inner.seed_statistics().num_periods
    }

    pub fn num_scenarios(&self) -> usize {
        self.inner.scenario_results.len()
    }

    pub fn num_observations(&self) -> usize {
        self.inner.distribution.len()
    }

    pub fn free_flow_travel_time_min(&self) -> f64 {
        self.inner.free_flow_travel_time_min
    }

    pub fn expected_vhd(&self) -> f64 {
        self.inner.expected_vhd
    }

    pub fn tti_mean(&self) -> f64 {
        self.inner.distribution.mean()
    }

    /// Weighted percentile TTI (p in 0-100), e.g. 95 for the PTI.
    pub fn tti_percentile(&self, p: f64) -> f64 {
        self.inner.distribution.percentile(p)
    }

    /// Misery index (mean of the worst 5% of TTIs).
    pub fn misery_index(&self) -> f64 {
        self.inner.distribution.misery_index()
    }

    /// Reliability rating, % (weighted share with TTI < 1.33).
    pub fn reliability_rating(&self) -> f64 {
        self.inner.distribution.reliability_rating()
    }

    /// Semi-standard deviation (one-sided about TTI = 1).
    pub fn semi_std_dev(&self) -> f64 {
        self.inner.distribution.semi_std_dev()
    }

    /// Percentage of the weighted distribution below the target facility
    /// space mean speed, %.
    pub fn failure_pct_below_speed(&self, target_speed_mi_h: f64) -> f64 {
        self.inner.failure_pct_below_speed(target_speed_mi_h)
    }

    /// Scenario probabilities (one entry per generated scenario).
    pub fn scenario_probabilities(&self) -> Vec<f64> {
        self.inner
            .scenario_results
            .iter()
            .map(|r| r.probability)
            .collect()
    }

    /// Month of year (1-12) of each scenario's demand combination. Empty
    /// before `run()`. This and the four vectors below share the ordering of
    /// `scenario_probabilities()` and `scenario_tti_matrix()`, so a scenario
    /// is identified by its index across all of them.
    pub fn scenario_months(&self) -> Vec<u32> {
        self.scenarios().iter().map(|sc| sc.month).collect()
    }

    /// Day of week of each scenario, as the English weekday name.
    pub fn scenario_weekdays(&self) -> Vec<String> {
        self.scenarios()
            .iter()
            .map(|sc| format!("{:?}", sc.weekday))
            .collect()
    }

    /// Demand adjustment factor DAF_s of each scenario (Equation 25-72),
    /// the scenario's demand multiplier over the seed date's. The seed-date
    /// scenario therefore has DAF = 1 by construction.
    pub fn scenario_dafs(&self) -> Vec<f64> {
        self.scenarios().iter().map(|sc| sc.daf).collect()
    }

    /// Number of incidents assigned to each scenario. Scenarios with zero
    /// incidents differ from the seed only through demand, which is what
    /// makes them comparable against a plain Chapter 10 run.
    pub fn scenario_incident_counts(&self) -> Vec<u32> {
        self.scenarios()
            .iter()
            .map(|sc| sc.incidents.len() as u32)
            .collect()
    }

    /// Expected incident frequency n_j per study period by month, indexed
    /// January = 0 (Equation 25-77). Months outside the reliability
    /// reporting period read zero. Empty before `run()`.
    pub fn monthly_incident_frequencies(&self) -> Vec<f64> {
        self.inner
            .scenario_set
            .as_ref()
            .map(|s| s.monthly_incident_frequency.clone())
            .unwrap_or_default()
    }

    /// Total incidents generated across the whole scenario set. The count
    /// is a draw, not the expectation, so it moves with the rng seed.
    pub fn total_incidents(&self) -> usize {
        self.inner
            .scenario_set
            .as_ref()
            .map(|s| s.total_incidents)
            .unwrap_or(0)
    }

    /// Per-scenario TTI matrix [scenario][period].
    pub fn scenario_tti_matrix(&self) -> JsValue {
        let tti: Vec<Vec<f64>> = self
            .inner
            .scenario_results
            .iter()
            .map(|r| r.tti.clone())
            .collect();
        serde_wasm_bindgen::to_value(&tti).unwrap_or(JsValue::NULL)
    }

    pub fn results_to_js_value(&self) -> JsValue {
        let obj = js_sys::Object::new();
        js_sys::Reflect::set(&obj, &JsValue::from_str("num_scenarios"), &JsValue::from(self.num_scenarios() as u32)).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("num_observations"), &JsValue::from(self.num_observations() as u32)).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("free_flow_travel_time_min"), &JsValue::from(self.free_flow_travel_time_min())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("tti_mean"), &JsValue::from(self.tti_mean())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("tti_50"), &JsValue::from(self.tti_percentile(50.0))).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("tti_80"), &JsValue::from(self.tti_percentile(80.0))).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("tti_95"), &JsValue::from(self.tti_percentile(95.0))).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("misery_index"), &JsValue::from(self.misery_index())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("reliability_rating"), &JsValue::from(self.reliability_rating())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("semi_std_dev"), &JsValue::from(self.semi_std_dev())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("expected_vhd"), &JsValue::from(self.expected_vhd())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("total_incidents"), &JsValue::from(self.total_incidents() as u32)).unwrap();

        JsValue::from(obj)
    }
}

impl WasmFreewayReliability {
    fn scenarios(&self) -> &[FreewayScenario] {
        self.inner
            .scenario_set
            .as_ref()
            .map(|s| s.scenarios.as_slice())
            .unwrap_or(&[])
    }
}
