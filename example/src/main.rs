use catacombs_macros::channel;
// #[channel(ExampleTx, BallsRx)]
// pub struct Test {
//
//
// }
use catacombs_macros::connections;

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

fn main() {}
