pub use crate::message::submessage::element::Timestamp;
pub use crate::rtps::cache::InstanceHandle;
use crate::DdsData;
use speedy::{Endianness, Readable};

/// DataSample represents an atom of data information (i.e., one value for one instance) as returned by DataReader’s read/take operations
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

/// Whether the sample has been read by a DataReader.
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum SampleStateKind {
    Read,
    NotRead,
}

/// The state of an instance.
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum InstanceStateKind {
    /// The instance is alive.
    Alive,
    /// The instance has been disposed.
    NotAliveDisposed,
    /// All writers that wrote to the instance have stopped writing.
    NotAliveNoWriters,
}

/// Whether the instance has been viewed by a DataReader.
///
/// Once the instance's state becomes NotAlive, this is reset
/// to New.
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ViewStateKind {
    New,
    NotNew,
}

/// Information about a sample.
#[derive(PartialEq, Clone, Copy, Debug)]
pub struct SampleInfo {
    /// Whether the data has been read.
    /// Read if it has been read at least once;
    /// NotRead otherwise.
    pub sample_state: SampleStateKind,
    /// Whether the instance has been read.
    pub view_state: ViewStateKind,
    /// The state of the instance.
    pub instance_state: InstanceStateKind,
    /// Whether `DataSample`'s data contains a valid sample.
    pub valid_data: bool,
    /// The timestamp when the data was sent.
    pub source_timestamp: Timestamp,
    /// The identifier of the instance.
    pub instance_handle: InstanceHandle,
    /// The number of times the instance's state changed from NotAliveDisposed to Alive.
    pub disposed_generation_count: i32,
    /// The number of times the instance's state changed from NotAliveNoWriters to Alive.
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
