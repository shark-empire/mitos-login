use std::time::SystemTime;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SessionState {
    Starting,
    Running,
    Ended,
}

#[derive(Clone, Debug)]
pub struct Session {
    pub id: u32,
    pub user: String,
    pub uid: u32,
    pub seat: String,
    pub pid: i32,
    pub started_at: SystemTime,
    pub state: SessionState,
}
