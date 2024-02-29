pub const GAMES_PER_MATCH: usize = 2_500;
pub const EPOCHS: usize = 4; // ?
pub const MINIBATCH_SIZE: usize = 12_000;
pub const LEARNING_RATE: f32 = 0.0008;

pub const DEPTH_PER_MOVE: u8 = 10;
pub const PERC_CHANCE_FOR_RANDOM_MOVE: u8 = 1;
pub const CONCURRENT_GAMES: usize = 4;
pub const MAX_PLY: usize = 300;

pub const INPUT_NODES: usize = 768;
pub const HIDDEN_NODES: usize = 64;
pub const OUTPUT_NODES: usize = 1;

pub const BUCKETS: usize = 1;