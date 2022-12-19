#[derive(Debug, Clone, Copy)]
pub enum Attribute {
    /// Filename
    Name,
    /// Absolute path
    Path,
    /// Path of the subvolume this is a snapshot of
    ParentPath,
    /// Creation time
    Created,
    Uuid,
}
