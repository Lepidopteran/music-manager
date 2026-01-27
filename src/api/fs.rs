use ts_rs::TS;

#[derive(Debug, Clone, Default, Eq, PartialEq, serde::Serialize, TS)]
pub enum FileSystemOperationStatus {
    #[default]
    Pending,
    InProgress,
    Completed,
    Cancelled,
    Failed,
}
