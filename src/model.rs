use bollard::container::Stats;

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub container_id: String,
    pub container_name: String,
    pub ts: i64,
    pub pid_count: Option<u64>,
    pub pid_limit: Option<u64>,
    pub memory_usage: Option<u64>,
    pub memory_limit: Option<u64>,
    pub cpu_percent: f64,
    pub cpu_count: u64,
    pub cpu_energy: Option<f64>,
}

impl From<Stats> for Snapshot {
    fn from(item: Stats) -> Self {
        let cpu_delta =
            item.cpu_stats.cpu_usage.total_usage - item.precpu_stats.cpu_usage.total_usage;
        let system_delta = item.cpu_stats.system_cpu_usage.unwrap_or_default()
            - item.precpu_stats.system_cpu_usage.unwrap_or_default();
        let cpu_count = item.cpu_stats.online_cpus.unwrap_or(1);
        let cpu_percent = cpu_delta as f64 / system_delta as f64;

        Self {
            container_id: item.id,
            container_name: item.name.trim_start_matches('/').to_string(),
            ts: item.read.timestamp(),
            pid_count: item.pids_stats.current,
            pid_limit: item.pids_stats.limit,
            memory_usage: item.memory_stats.usage,
            memory_limit: item.memory_stats.limit,
            cpu_percent,
            cpu_count,
            cpu_energy: None,
        }
    }
}

impl Snapshot {
    pub fn with_energy(mut self, total_cpu_energy: Option<u64>) -> Self {
        if let Some(total_cpu_energy) = total_cpu_energy {
            self.cpu_energy = Some(self.cpu_percent * total_cpu_energy as f64);
        }
        self
    }
}
