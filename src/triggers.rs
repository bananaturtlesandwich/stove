use super::*;

// actions
#[derive(Event)]
pub struct Duplicate;
#[derive(Event)]
pub struct Delete;
#[derive(Event)]
pub struct Focus;
#[derive(Event)]
pub struct Copy;
#[derive(Event)]
pub struct Paste;
#[derive(Event)]
pub struct Deselect;
#[derive(Event)]
pub struct Fullscreen;
#[derive(Event)]
pub struct Hide;
#[derive(Event)]
pub struct LoadPaks;

// dialogs
#[derive(Event)]
pub struct Open(pub Option<std::path::PathBuf>);
#[derive(Event)]
pub struct SaveAs(pub bool);
#[derive(Event)]
pub struct AddPak;
#[derive(Event)]
pub struct Transplant;
