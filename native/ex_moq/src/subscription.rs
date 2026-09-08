use rustler::NifStruct;

#[derive(NifStruct, Clone)]
#[module = "ExMoQ.Subscription"]
pub(crate) struct Subscription {
    pub priority: Option<u8>,
    pub group_start: Option<u64>,
    pub latency_ns: Option<u64>,
}
