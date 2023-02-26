pub trait MpscChannel<S, R>
{
    fn create_channel(buffer: usize) -> (S, R);
}

pub trait MpmcChannel<S, R>
{
    fn create_channel(buffer: usize) -> (S, R);
}


