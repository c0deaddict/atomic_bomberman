pub const STAGE: &str = "app_state";

#[derive(Clone)]
pub enum AppState {
    Loading,
    Menu,
    // InGame
}
