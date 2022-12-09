use std::ops::{AddAssign, DivAssign};
use std::time::Duration;

#[derive(Debug, Default)]
pub struct Metric {
    pub time_elapsed: Duration,
    pub txs_sent: u64,
    pub txs_confirmed: u64,
    pub txs_un_confirmed: u64,
}

#[derive(Default)]
pub struct AvgMetric {
    num_of_runs: u64,
    total_metric: Metric,
}

impl Metric {
    pub fn calc_tps(&self) -> f64 {
        todo!()
    }
}

impl AddAssign for Metric {
    fn add_assign(&mut self, rhs: Self) {
        self.time_elapsed += rhs.time_elapsed;
        self.txs_sent += rhs.txs_sent;
        self.txs_confirmed += rhs.txs_confirmed;
        self.txs_un_confirmed += rhs.txs_un_confirmed;
    }
}

impl DivAssign<u64> for Metric {
    fn div_assign(&mut self, rhs: u64) {
        self.time_elapsed = Duration::from_nanos(self.time_elapsed.as_nanos() as u64 / rhs);
        self.txs_sent /= rhs;
        self.txs_confirmed /= rhs;
        self.txs_un_confirmed /= rhs;
    }
}

impl AddAssign<Metric> for AvgMetric {
    fn add_assign(&mut self, rhs: Metric) {
        self.num_of_runs += 1;
        self.total_metric += rhs;
    }
}

impl From<AvgMetric> for Metric {
    fn from(mut avg_metric: AvgMetric) -> Self {
        avg_metric.total_metric /= avg_metric.num_of_runs;
        avg_metric.total_metric
    }
}
