use rustler::NifStruct;

#[derive(NifStruct, Clone, Copy)]
#[module = "ExMoQ.Subscription"]
pub(crate) struct Subscription {
    pub priority: Option<u8>,
    pub group_start: Option<u64>,
    pub latency_ns: u64,
}
