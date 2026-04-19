use std::time::Duration;

pub struct BenchResult {
    durations: Vec<Duration>,
}

impl BenchResult {
    pub fn new(durations: Vec<Duration>) -> Self {
        Self { durations }
    }

    pub fn p50_ms(&self) -> f64 { self.percentile_ns(0.50) / 1e6 }
    pub fn p99_ms(&self) -> f64 { self.percentile_ns(0.99) / 1e6 }
    pub fn p99_secs(&self) -> f64 { self.percentile_ns(0.99) / 1e9 }
    pub fn p50_us(&self) -> f64 { self.percentile_ns(0.50) / 1e3 }
    pub fn p99_us(&self) -> f64 { self.percentile_ns(0.99) / 1e3 }
    pub fn mean_us(&self) -> f64 { self.mean_ns() / 1e3 }
    pub fn std_us(&self) -> f64 { self.std_ns() / 1e3 }
    pub fn mean_ms(&self) -> f64 { self.mean_ns() / 1e6 }
    pub fn std_ms(&self) -> f64 { self.std_ns() / 1e6 }
    pub fn total_secs(&self) -> f64 { self.durations.iter().map(|d| d.as_secs_f64()).sum() }
    pub fn count(&self) -> usize { self.durations.len() }

    pub fn std_ratio(&self) -> f64 {
        if self.durations.is_empty() || self.mean_ns() == 0.0 { return 0.0; }
        self.std_ns() / self.mean_ns()
    }

    fn mean_ns(&self) -> f64 {
        if self.durations.is_empty() { return 0.0; }
        self.durations.iter().map(|d| d.as_nanos() as f64).sum::<f64>() / self.durations.len() as f64
    }

    fn std_ns(&self) -> f64 {
        if self.durations.is_empty() { return 0.0; }
        let mean = self.mean_ns();
        let variance = self.durations.iter()
            .map(|d| (d.as_nanos() as f64 - mean).powi(2))
            .sum::<f64>() / self.durations.len() as f64;
        variance.sqrt()
    }

    fn percentile_ns(&self, p: f64) -> f64 {
        if self.durations.is_empty() { return 0.0; }
        let mut ns: Vec<f64> = self.durations.iter().map(|d| d.as_nanos() as f64).collect();
        ns.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let idx = (p * (ns.len() - 1) as f64).round() as usize;
        ns[idx]
    }
}