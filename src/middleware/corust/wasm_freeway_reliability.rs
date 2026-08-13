use wasm_bindgen::prelude::*;
use transportations_library::hcm::chapter11::reliability::ReliabilityAnalysis;
use transportations_library::hcm::chapter11::exhibits::{INCIDENT_SEVERITIES, SEVERE_WEATHER_TYPES};
use transportations_library::hcm::chapter11::scenario_generation::{
    FreewayScenario, IncidentInputs, Weekday, WeatherInputs,
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
/// scoped to demand variability plus optional weather and incidents. The
/// scenario generator defaults to a whole-year reliability reporting period
/// (12 months, Monday through Friday, Exhibit 11-18 urban demand ratios) with
/// no weather; `set_weather()` and `set_demand_multipliers()` replace those
/// two defaults, and `set_incidents()` replaces the incident inputs the
/// constructor's two scalars can only partly describe. Work zones and special
/// events, and with them the Chapter 37 ATDM strategies built on top of them,
/// are not exposed by this binding.
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct WasmFreewayReliability {
    inner: ReliabilityAnalysis,
}

#[wasm_bindgen]
impl WasmFreewayReliability {

    /// The four trailing facility parameters are the ones
    /// `WasmFreewayFacility` has always taken and this constructor used to
    /// pass as `None`: jam density, the queue discharge capacity drop, total
    /// ramp density, and interchange density. Omitting any of them keeps the
    /// core default, which for the first three is the value Example Problem 7
    /// itself uses (190 pc/mi/ln, 7%, 1.0 ramps/mi) but for interchange
    /// density is `None`, and the core then falls back to the total ramp
    /// density rather than to Example Problem 7's 0.8 interchanges/mi.
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
        jam_density_pc: Option<f64>,
        queue_discharge_drop: Option<f64>,
        total_ramp_density: Option<f64>,
        interchange_density: Option<f64>,
    ) -> Self {
        let facility = build_facility(
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

    /// Place the Step B-6 weather inputs, from a configuration object in the
    /// serde schema of the library's `WeatherInputs`, so the `weather` object
    /// of `tests/ExampleCases/hcm/FreewayReliability/case1.json` passes
    /// verbatim. The inputs are a 12-by-10 timewise probability matrix
    /// (Equation 25-75, months by `SEVERE_WEATHER_TYPES`), the ten mean event
    /// durations, optional per-type CAF and SAF overrides on the Exhibit
    /// 11-20/11-21 defaults, and the event demand adjustment factor, so they
    /// arrive as one config object rather than as a further row of positional
    /// arguments. Without this the generator sees no weather at all, which is
    /// a milder facility rather than a differently parameterized one.
    ///
    /// The shapes are checked here because `WeatherInputs` is `serde(default)`:
    /// a misspelled or transposed probability matrix would otherwise
    /// deserialize into the all-zero default and silently generate the same
    /// weather-free distribution the caller was trying to leave behind.
    pub fn set_weather(&mut self, config: JsValue) -> Result<(), JsValue> {
        let weather: WeatherInputs = serde_wasm_bindgen::from_value(config)
            .map_err(|e| JsValue::from_str(&format!("Invalid weather config: {e}")))?;
        let n_types = SEVERE_WEATHER_TYPES.len();
        if weather.probabilities_by_month.len() != 12 {
            return Err(JsValue::from_str(&format!(
                "weather.probabilities_by_month must have 12 rows (January-December), got {}",
                weather.probabilities_by_month.len()
            )));
        }
        for (i, row) in weather.probabilities_by_month.iter().enumerate() {
            if row.len() != n_types {
                return Err(JsValue::from_str(&format!(
                    "weather.probabilities_by_month row {} must have {n_types} entries (SEVERE_WEATHER_TYPES order), got {}",
                    i + 1,
                    row.len()
                )));
            }
        }
        if weather.durations_min.len() != n_types {
            return Err(JsValue::from_str(&format!(
                "weather.durations_min must have {n_types} entries, got {}",
                weather.durations_min.len()
            )));
        }
        for (name, over) in [
            ("caf_override", &weather.caf_override),
            ("saf_override", &weather.saf_override),
        ] {
            if let Some(v) = over {
                if v.len() != n_types {
                    return Err(JsValue::from_str(&format!(
                        "weather.{name} must have {n_types} entries, got {}",
                        v.len()
                    )));
                }
            }
        }
        self.inner.scenario_generation.weather = Some(weather);
        Ok(())
    }

    /// Remove the weather inputs, returning the generator to its default of
    /// modeling no weather events.
    pub fn clear_weather(&mut self) {
        self.inner.scenario_generation.weather = None;
    }

    pub fn has_weather(&self) -> bool {
        self.inner.scenario_generation.weather.is_some()
    }

    /// Place the Step B-7 incident inputs, from a configuration object in the
    /// serde schema of the library's `IncidentInputs`, so the `incidents`
    /// object of `tests/ExampleCases/hcm/FreewayReliability/case1.json` passes
    /// verbatim. This replaces whatever the constructor's `crash_rate_per_100mvmt`
    /// and `incident_to_crash_ratio` arguments built, which is an
    /// `IncidentInputs::default()` carrying those two scalars and the national
    /// defaults for everything else. Three of those defaults are analysis
    /// inputs in their own right and had no way in: the Equation 25-77 monthly
    /// frequencies when local counts are known rather than a crash rate, the
    /// Equation 25-85 severity distribution G(i), and the Exhibit 11-22
    /// lognormal duration parameters by severity. The last of those is what
    /// Chapter 25 Example Problem 9 changes, cutting every severity's mean
    /// duration and standard deviation by 30% to model improved incident
    /// management, so before this setter the published incident-management
    /// alternative could not be expressed through the binding at all.
    ///
    /// The shapes are checked here because `IncidentInputs` is `serde(default)`
    /// and the core reads all three leniently. `duration_params` is looked up
    /// as `get(severity).unwrap_or(Exhibit 11-22 default)`, so a four-entry
    /// list quietly restores the book durations for the severity it omits,
    /// which is the exact mistake a 30% cut written out by hand invites.
    /// `severity_distribution` is validated by the core for its sum but not
    /// its length, so a short one drops severities and a long one indexes past
    /// the five of Equation 25-85. `monthly_frequencies` is read as
    /// `get(month).unwrap_or(0.0)`, so an eleven-month table silently models a
    /// year with one incident-free month. Finally, an incident configuration
    /// with neither `monthly_frequencies` nor `crash_rate_per_100mvmt` has no
    /// frequency source at all and generates zero incidents while reporting
    /// that incidents are modeled, which is what a wholly misspelled config
    /// deserializes into, so it is rejected rather than run.
    pub fn set_incidents(&mut self, config: JsValue) -> Result<(), JsValue> {
        let incidents: IncidentInputs = serde_wasm_bindgen::from_value(config)
            .map_err(|e| JsValue::from_str(&format!("Invalid incident config: {e}")))?;
        let n_sev = INCIDENT_SEVERITIES.len();
        if incidents.severity_distribution.len() != n_sev {
            return Err(JsValue::from_str(&format!(
                "incidents.severity_distribution must have {n_sev} entries (Equation 25-85 order: shoulder, 1, 2, 3, 4+ lanes closed), got {}",
                incidents.severity_distribution.len()
            )));
        }
        if incidents.duration_params.len() != n_sev {
            return Err(JsValue::from_str(&format!(
                "incidents.duration_params must have {n_sev} entries (one per severity, Equation 25-85 order), got {}",
                incidents.duration_params.len()
            )));
        }
        if let Some(freqs) = &incidents.monthly_frequencies {
            if freqs.len() != 12 {
                return Err(JsValue::from_str(&format!(
                    "incidents.monthly_frequencies must have 12 entries (January-December), got {}",
                    freqs.len()
                )));
            }
        }
        if incidents.monthly_frequencies.is_none() && incidents.crash_rate_per_100mvmt.is_none() {
            return Err(JsValue::from_str(
                "incidents needs a frequency source: set monthly_frequencies (Equation 25-77 n_j) or crash_rate_per_100mvmt (Equation 25-78), or clear_incidents() to model none",
            ));
        }
        self.inner.scenario_generation.incidents = Some(incidents);
        Ok(())
    }

    /// Remove the incident inputs, returning the generator to modeling no
    /// incidents at all. This is the state a constructor called without a
    /// crash rate leaves behind.
    pub fn clear_incidents(&mut self) {
        self.inner.scenario_generation.incidents = None;
    }

    pub fn has_incidents(&self) -> bool {
        self.inner.scenario_generation.incidents.is_some()
    }

    /// The Exhibit 11-22 lognormal duration parameters as the generator will
    /// use them, one object per severity in Equation 25-85 order, or `null`
    /// when no incidents are modeled. A caller scaling the book defaults (the
    /// Example Problem 9 alternative) reads them here rather than transcribing
    /// the exhibit, and a caller that supplied its own can confirm the list
    /// that arrived is the list that will run.
    pub fn incident_duration_params(&self) -> JsValue {
        self.inner
            .scenario_generation
            .incidents
            .as_ref()
            .and_then(|inc| serde_wasm_bindgen::to_value(&inc.duration_params).ok())
            .unwrap_or(JsValue::NULL)
    }

    /// Replace the demand multipliers DM of Equation 25-72 with a local
    /// table: 12 rows (January through December) of 7 columns (Monday through
    /// Sunday). Only ratios to the seed date's multiplier are used, so any
    /// common base works, which is why Example Problem 7's Exhibit 25-100
    /// table (an ADT-based rescaling of Exhibit 11-18) gives a July Friday DAF
    /// of 1.329/0.995 = 1.3357 where the Exhibit 11-18 default gives
    /// 1.62/1.21 = 1.3388.
    ///
    /// The shape is checked because the core's lookup returns 1.0 for a month
    /// or weekday the table does not reach, so a transposed or short table
    /// would quietly flatten part of the year to no demand variation at all.
    pub fn set_demand_multipliers(&mut self, rows: JsValue) -> Result<(), JsValue> {
        let rows: Vec<Vec<f64>> = serde_wasm_bindgen::from_value(rows)
            .map_err(|e| JsValue::from_str(&format!("Invalid demand multipliers: {e}")))?;
        if rows.len() != 12 {
            return Err(JsValue::from_str(&format!(
                "demand multipliers must have 12 rows (January-December), got {}",
                rows.len()
            )));
        }
        for (i, row) in rows.iter().enumerate() {
            if row.len() != 7 {
                return Err(JsValue::from_str(&format!(
                    "demand multiplier row {} must have 7 entries (Monday-Sunday), got {}",
                    i + 1,
                    row.len()
                )));
            }
        }
        self.inner.scenario_generation.demand_multipliers = rows;
        Ok(())
    }

    /// Demand multiplier DM(Seed) of the seed dataset date, the denominator
    /// of every scenario's DAF (Equation 25-72).
    pub fn seed_demand_multiplier(&self) -> f64 {
        self.inner.scenario_generation.seed_demand_multiplier()
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

    /// Largest TTI in the weighted distribution. This is the single worst
    /// scenario-period, so it is the measure most exposed to the Monte Carlo
    /// pairing of an incident with a high-demand scenario.
    pub fn tti_max(&self) -> f64 {
        self.inner.distribution.max()
    }

    /// Percentage of the weighted distribution above a TTI threshold, e.g.
    /// 2.0 for the Exhibit 25-104 "%VMT at TTI > 2" row.
    pub fn pct_tti_above(&self, threshold: f64) -> f64 {
        self.inner.distribution.pct_above(threshold)
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

    /// Percentage of the weighted distribution at or above the target
    /// facility space mean speed, %. The complement of
    /// `failure_pct_below_speed()` at the same target.
    pub fn on_time_pct_at_speed(&self, target_speed_mi_h: f64) -> f64 {
        self.inner.on_time_pct_at_speed(target_speed_mi_h)
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

    /// Expected weather event counts E[n_w,j] by month (12 rows, January
    /// first) and severe weather type (10 columns, `SEVERE_WEATHER_TYPES`
    /// order), Equation 25-76. These are the deterministic counts the
    /// stochastic assignment then places, so they are the part of the
    /// weather step that reproduces exactly. Empty before `run()`.
    pub fn expected_weather_event_counts(&self) -> JsValue {
        let counts: Vec<Vec<u32>> = self
            .inner
            .scenario_set
            .as_ref()
            .map(|s| s.expected_weather_events.clone())
            .unwrap_or_default();
        serde_wasm_bindgen::to_value(&counts).unwrap_or(JsValue::NULL)
    }

    /// Total weather events generated across the whole scenario set.
    pub fn total_weather_events(&self) -> usize {
        self.inner
            .scenario_set
            .as_ref()
            .map(|s| s.total_weather_events)
            .unwrap_or(0)
    }

    /// Number of weather events assigned to each scenario, sharing the
    /// ordering of `scenario_probabilities()`.
    pub fn scenario_weather_event_counts(&self) -> Vec<u32> {
        self.scenarios()
            .iter()
            .map(|sc| sc.weather_events.len() as u32)
            .collect()
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
