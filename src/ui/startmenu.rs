#[derive(Clone)]
pub enum StartMenuMessage {
    ChoosePlayerCount(u32),
    Start,
}
