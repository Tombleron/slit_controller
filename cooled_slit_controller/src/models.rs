use em2rs::Em2rsState;

type AxisStateValue<T> = Result<T, String>;

#[derive(Debug)]
pub struct AxisState {
    pub position: AxisStateValue<f32>,
    pub temperature: AxisStateValue<f32>,
    pub state: AxisStateValue<Em2rsState>,
    pub is_moving: AxisStateValue<bool>,
}

pub struct SharedState {
    pub axes: [Option<AxisState>; 4],
}
