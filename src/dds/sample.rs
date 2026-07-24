use crate::message::submessage::element::Timestamp;
use crate::rtps::cache::InstanceHandle;
use crate::DdsData;
use speedy::{Endianness, Readable};

pub struct DataSample<R: for<'a> Readable<'a, Endianness> + DdsData> {
    data: R,
    sample_info: SampleInfo,
}

impl<R: for<'a> Readable<'a, Endianness> + DdsData> DataSample<R> {
    pub(crate) fn new(data: R, sample_info: SampleInfo) -> Self {
        Self { data, sample_info }
    }

    pub fn data(&self) -> &R {
        &self.data
    }

    pub fn sample_info(&self) -> &SampleInfo {
        &self.sample_info
    }
}

pub struct SampleInfo {
    pub source_timestamp: Timestamp,
    pub instance_handle: InstanceHandle,
}

impl SampleInfo {
    pub(crate) fn new(source_ts: Timestamp, instance_handle: InstanceHandle) -> Self {
        Self {
            source_timestamp: source_ts,
            instance_handle,
        }
    }
}
