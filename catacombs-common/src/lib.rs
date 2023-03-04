pub mod parser;

use tokio::sync::{
    broadcast::{channel as mpmc_channel, Receiver as MpmcReceiver, Sender as MpmcSender},
    mpsc::{channel as mpsc_channel, Receiver as MpscReceiver, Sender as MpscSender}
};

pub trait Channel<S, R>
{
    fn create_channel(buffer: usize) -> (S, R);
}

pub struct TokioMpscChannel;
pub struct TokioMpmcChannel;

impl<T: Send + Clone> Channel<MpscSender<T>, MpscReceiver<T>> for TokioMpscChannel
{
    fn create_channel(buffer: usize) -> (MpscSender<T>, MpscReceiver<T>)
    {
        mpsc_channel(buffer)
    }
}

impl<T: Send + Clone> Channel<MpmcSender<T>, MpmcReceiver<T>> for TokioMpmcChannel
{
    fn create_channel(buffer: usize) -> (MpmcSender<T>, MpmcReceiver<T>)
    {
        mpmc_channel(buffer)
    }
}
