use wasm_bindgen::prelude::*;
use transportations_library::hcm::chapter17::exhibits::{
    FunctionalClass, URBAN_RELIABILITY_RATING_TTI_THRESHOLD,
};
use transportations_library::urban_facilities::UrbanFacility;
use transportations_library::urban_reliability::{
    BoundarySignal, MonthlyWeather, UrbanReliability, UrbanReliabilityConfig,
};
use transportations_library::urban_segments::{BoundaryControlType, UrbanSegment};

/// Value of a monthly weather statistic for month index `i` (0 = January).
/// Empty input means 0.0 for every month (no events of that kind); a single
/// value is applied to all 12 months; 12 values are used as given.
fn month_value(values: &[f64], i: usize) -> f64 {
    if values.is_empty() {
        0.0
    } else {
        values[i.min(values.len() - 1)]
    }
}

fn parse_functional_class(class: &str) -> FunctionalClass {
    match class.to_lowercase().as_str() {
        "expressway" => FunctionalClass::Expressway,
        "minor" | "urbanminorarterial" | "minor_arterial" => FunctionalClass::UrbanMinorArterial,
        _ => FunctionalClass::UrbanPrincipalArterial,
    }
}

#[wasm_bindgen]
pub struct WasmUrbanReliability {
    config: UrbanReliabilityConfig,
    segments: Vec<UrbanSegment>,
    inner: Option<UrbanReliability>,
}

#[wasm_bindgen]
impl WasmUrbanReliability {

    /// Scope: a fully signalized urban street facility (every segment's
    /// downstream boundary intersection is a traffic signal), evaluated
    /// with the HCM default demand-ratio, weather, and incident models.
    /// Each monthly weather array takes 0, 1, or 12 entries (none, one
    /// value replicated to every month, or January-December values).
    #[wasm_bindgen(constructor)]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        functional_class: Option<String>,
        study_period_start_hour: Option<u32>,
        analysis_periods_per_day: Option<u32>,
        monthly_total_precip_in: Vec<f64>,
        monthly_days_with_precip: Vec<f64>,
        monthly_mean_temp_f: Vec<f64>,
        monthly_precip_rate_in_h: Vec<f64>,
        entry_intersection_crash_frequency: Option<f64>,
        minor_leg_volume_veh_h: Option<f64>,
        shoulder_present: Option<bool>,
        vmt_weighted: Option<bool>,
        weather_seed: Option<u32>,
        demand_seed: Option<u32>,
        incident_seed: Option<u32>,
    ) -> Self {
        let mut config = UrbanReliabilityConfig::default();
        if let Some(c) = functional_class {
            config.functional_class = parse_functional_class(&c);
        }
        if let Some(v) = study_period_start_hour {
            config.study_period_start_hour = v.min(23);
            config.count_hour = v.min(23);
        }
        if let Some(v) = analysis_periods_per_day {
            config.analysis_periods_per_day = v as usize;
        }
        config.weather = (0..12)
            .map(|i| MonthlyWeather {
                total_precip_in: month_value(&monthly_total_precip_in, i),
                total_snowfall_in: 0.0,
                days_with_precip: month_value(&monthly_days_with_precip, i),
                mean_temp_f: month_value(&monthly_mean_temp_f, i),
                precip_rate_in_h: month_value(&monthly_precip_rate_in_h, i),
            })
            .collect();
        config.incidents.intersection_crash_frequencies =
            vec![entry_intersection_crash_frequency.unwrap_or(0.0)];
        if let Some(v) = minor_leg_volume_veh_h {
            config.incidents.minor_leg_volume_veh_h = v;
        }
        if let Some(v) = shoulder_present {
            config.incidents.shoulder_present = v;
        }
        if let Some(v) = vmt_weighted {
            config.vmt_weighted = v;
        }
        if let Some(v) = weather_seed {
            config.weather_seed = v as u64;
        }
        if let Some(v) = demand_seed {
            config.demand_seed = v as u64;
        }
        if let Some(v) = incident_seed {
            config.incident_seed = v as u64;
        }
        WasmUrbanReliability {
            config,
            segments: Vec::new(),
            inner: None,
        }
    }

    /// Append a signalized Chapter 18 segment (ordered upstream to
    /// downstream) with its boundary-signal timing and the crash
    /// frequencies used by the incident generator. The intersection crash
    /// frequency belongs to the segment's downstream boundary
    /// intersection.
    #[allow(clippy::too_many_arguments)]
    pub fn add_segment(
        &mut self,
        segment_length_ft: f64,
        n_through_lanes: u32,
        speed_limit_mph: f64,
        through_demand_veh_h: f64,
        cycle_length_s: f64,
        effective_green_s: f64,
        sat_flow_veh_h_ln: Option<f64>,
        platoon_ratio: Option<f64>,
        n_access_points_subject: Option<f64>,
        n_access_points_opposing: Option<f64>,
        full_stop_rate_override: Option<f64>,
        segment_crash_frequency: f64,
        intersection_crash_frequency: f64,
    ) {
        let sat_flow = sat_flow_veh_h_ln.unwrap_or(1_800.0);
        let r_p = platoon_ratio.unwrap_or(1.0);
        let mut segment = UrbanSegment::new(
            segment_length_ft,
            n_through_lanes,
            speed_limit_mph,
            through_demand_veh_h,
            BoundaryControlType::Signalized,
        );
        segment.midsegment_flow_veh_h = Some(through_demand_veh_h);
        segment.cycle_length_s = Some(cycle_length_s);
        segment.effective_green_s = Some(effective_green_s);
        segment.sat_flow_veh_h_ln = Some(sat_flow);
        segment.platoon_ratio = Some(r_p);
        if let Some(v) = n_access_points_subject {
            segment.n_access_points_subject = v;
        }
        if let Some(v) = n_access_points_opposing {
            segment.n_access_points_opposing = v;
        }
        if full_stop_rate_override.is_some() {
            segment.full_stop_rate_override = full_stop_rate_override;
        }
        self.segments.push(segment);
        self.config.boundary_signals.push(BoundarySignal {
            cycle_length_s,
            effective_green_s,
            sat_flow_veh_h_ln: sat_flow,
            platoon_ratio: r_p,
            k_factor: 0.5,
            i_factor: 1.0,
            approach_lanes: 0, // defaults to the segment's through lanes
        });
        self.config
            .incidents
            .segment_crash_frequencies
            .push(segment_crash_frequency);
        self.config
            .incidents
            .intersection_crash_frequencies
            .push(intersection_crash_frequency);
    }

    /// Run the full HCM Ch.17 methodology: weather, demand, and incident
    /// scenario generation over a one-year reliability reporting period
    /// (weekdays), Chapter 16/18 evaluation of every scenario, and the
    /// travel time distribution summary.
    pub fn run(&mut self) -> Result<(), JsValue> {
        let facility = UrbanFacility::new(self.segments.clone());
        let mut analysis = UrbanReliability::new(facility, self.config.clone());
        analysis.run().map_err(|e| JsValue::from_str(&e))?;
        self.inner = Some(analysis);
        Ok(())
    }

    pub fn num_segments(&self) -> u32 {
        self.segments.len() as u32
    }

    pub fn num_scenarios(&self) -> u32 {
        self.inner
            .as_ref()
            .and_then(|a| a.results.as_ref())
            .map_or(0, |r| r.num_scenarios as u32)
    }

    pub fn num_weather_events(&self) -> u32 {
        self.inner.as_ref().map_or(0, |a| a.weather_events.len() as u32)
    }

    pub fn num_incidents(&self) -> u32 {
        self.inner.as_ref().map_or(0, |a| a.incidents.len() as u32)
    }

    pub fn get_base_free_flow_travel_time(&self) -> Option<f64> {
        self.inner
            .as_ref()
            .and_then(|a| a.results.as_ref())
            .map(|r| r.base_free_flow_travel_time_s)
    }

    pub fn get_mean_travel_time(&self) -> Option<f64> {
        self.inner
            .as_ref()
            .and_then(|a| a.results.as_ref())
            .map(|r| r.mean_travel_time_s)
    }

    pub fn get_total_vhd(&self) -> Option<f64> {
        self.inner
            .as_ref()
            .and_then(|a| a.results.as_ref())
            .map(|r| r.total_vhd)
    }

    /// Mean travel time index across the weighted scenario distribution.
    pub fn tti_mean(&self) -> f64 {
        self.inner.as_ref().map_or(0.0, |a| a.distribution.mean())
    }

    /// Weighted percentile TTI (p in 0-100), e.g. 95 for the planning
    /// time index.
    pub fn tti_percentile(&self, p: f64) -> f64 {
        self.inner
            .as_ref()
            .map_or(0.0, |a| a.distribution.percentile(p))
    }

    /// Urban street reliability rating, percent of the weighted
    /// distribution with TTI below 2.5.
    pub fn reliability_rating(&self) -> f64 {
        self.inner.as_ref().map_or(0.0, |a| {
            a.distribution
                .pct_at_or_below(URBAN_RELIABILITY_RATING_TTI_THRESHOLD)
        })
    }

    pub fn results_to_js_value(&self) -> JsValue {
        let obj = js_sys::Object::new();
        let results = self.inner.as_ref().and_then(|a| a.results.as_ref());
        let opt = |v: Option<f64>| v.map(JsValue::from).unwrap_or(JsValue::NULL);
        js_sys::Reflect::set(&obj, &JsValue::from_str("num_scenarios"), &JsValue::from(self.num_scenarios())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("num_weather_events"), &JsValue::from(self.num_weather_events())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("num_incidents"), &JsValue::from(self.num_incidents())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("base_free_flow_travel_time"), &opt(self.get_base_free_flow_travel_time())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("mean_travel_time"), &opt(self.get_mean_travel_time())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("tti_mean"), &JsValue::from(self.tti_mean())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("tti_50"), &JsValue::from(self.tti_percentile(50.0))).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("tti_80"), &JsValue::from(self.tti_percentile(80.0))).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("tti_95"), &JsValue::from(self.tti_percentile(95.0))).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("reliability_rating"), &JsValue::from(self.reliability_rating())).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("total_vhd"), &opt(self.get_total_vhd())).unwrap();
        js_sys::Reflect::set(
            &obj,
            &JsValue::from_str("pct_nondry_scenarios"),
            &opt(results.map(|r| r.pct_nondry_scenarios)),
        )
        .unwrap();

        JsValue::from(obj)
    }
}
