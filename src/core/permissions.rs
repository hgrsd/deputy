pub enum PermissionMode {
    Ask,
    ApprovedForId { command_id: String },
}