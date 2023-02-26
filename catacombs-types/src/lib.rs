pub trait Channel<S, R>
{
    fn create_channel(buffer: usize) -> (S, R);
}
