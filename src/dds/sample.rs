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

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum SampleStateKind {
    Read,
    NotRead,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum InstanceStateKind {
    Alive,
    NotAliveDisposed,
    NotAliveNoWriters,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ViewStateKind {
    New,
    NotNew,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct SampleInfo {
    pub sample_state: SampleStateKind,
    pub view_state: ViewStateKind,
    pub instance_state: InstanceStateKind,
    pub valid_data: bool,
    pub source_timestamp: Timestamp,
    pub instance_handle: InstanceHandle,
    pub disposed_generation_count: i32,
    pub no_writers_generation_count: i32,
}

impl SampleInfo {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        is_read: bool,
        viewed: bool,
        instance_state: InstanceStateKind,
        valid_data: bool,
        source_ts: Timestamp,
        instance_handle: InstanceHandle,
        disposed_generation_count: i32,
        no_writers_generation_count: i32,
    ) -> Self {
        Self {
            sample_state: if is_read {
                SampleStateKind::Read
            } else {
                SampleStateKind::NotRead
            },
            view_state: if viewed {
                ViewStateKind::NotNew
            } else {
                ViewStateKind::New
            },
            instance_state,
            valid_data,
            source_timestamp: source_ts,
            instance_handle,
            disposed_generation_count,
            no_writers_generation_count,
        }
    }
}
