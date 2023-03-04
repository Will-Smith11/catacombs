use catacombs_macros::{channel, connections, inject};

connections! {
    Test, mpsc, 10;
    Balls, mpmc, 10;
}

#[derive(Debug, Clone)]
pub enum Balls
{
    One,
    Two,
    Three
}

#[derive(Debug, Clone)]
pub enum Test
{
    One,
    Two,
    Three
}

#[channel(TestTx, BallsRx)]
struct ChannelTest {}

impl ChannelTest
{
    #[inject]
    pub fn new() -> Self
    {
        let (test_tx, balls_rx) = Self::get_channels();
    }
}

fn main() {}
