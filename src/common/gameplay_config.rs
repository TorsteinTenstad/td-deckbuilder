pub const STARTING_HAND_SIZE: i32 = 4;
pub const MAX_HAND_SIZE: usize = 10;
pub const STARTING_ENERGY: i32 = 0;

const BASE_SECONDS_TO_DRAW_CARD: f32 = 20.0;
const BASE_SECONDS_TO_GET_ENERGY: f32 = 7.0;
pub const CARD_DRAW_PER_SECOND: f32 = 1.0 / BASE_SECONDS_TO_DRAW_CARD;
pub const ENERGY_PER_SECOND: f32 = 1.0 / BASE_SECONDS_TO_GET_ENERGY;
