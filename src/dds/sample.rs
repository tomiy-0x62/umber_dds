use crate::message::submessage::element::Timestamp;
use crate::rtps::cache::InstanceHandle;
use crate::DdsData;
use speedy::{Endianness, Readable};

pub struct DataSample<R: for<'a> Readable<'a, Endianness> + DdsData> {
    data: Option<R>,
    sample_info: SampleInfo,
}

impl<R: for<'a> Readable<'a, Endianness> + DdsData> DataSample<R> {
    pub(crate) fn new(data: Option<R>, sample_info: SampleInfo) -> Self {
        Self { data, sample_info }
    }

    pub fn data(&self) -> Option<&R> {
        self.data.as_ref()
    }

    pub fn sample_info(&self) -> &SampleInfo {
        &self.sample_info
    }
}

pub enum SampleState {
    Read,
    NotRead,
}

pub enum InstanceState {
    Alive,
    NotAliveDisposed,
    NotAliveNoWriters,
}

pub enum ViewState {
    New,
    NotNew,
}

pub struct SampleInfo {
    pub sample_state: SampleState,
    // pub view_state: ViewState,
    // pub instance_state: InstanceState,
    pub source_timestamp: Timestamp,
    pub instance_handle: InstanceHandle,
}

impl SampleInfo {
    pub(crate) fn new(
        is_read: bool,
        source_ts: Timestamp,
        instance_handle: InstanceHandle,
    ) -> Self {
        Self {
            sample_state: if is_read {
                SampleState::Read
            } else {
                SampleState::NotRead
            },
            source_timestamp: source_ts,
            instance_handle,
        }
    }
}
