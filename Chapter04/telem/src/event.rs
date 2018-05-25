#[derive(Clone)]
pub enum Event {
    Telemetry(Telemetry),
    Flush,
}

#[derive(Debug, Clone)]
pub struct Telemetry {
    pub name: String,
    pub value: u32,
}
