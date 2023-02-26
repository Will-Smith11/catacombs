use proc_macro::TokenStream;

#[proc_macro]
/// Connections describes the information about the channels, such as what
/// channel connects to another channel. The type of channel that we want and
/// then also the buffer size on each channel.
///
/// The syntax for this is as follows:
///
/// `name, data; channel_type, buffer_size`
///
/// ```rust
/// connections! {
///     subscriptions, SubscriptionEnum; spsc, 10
///     incoming_http, HttpRequestsStuct; mpsc, 5
/// }
/// ```
///The types of channels that are supported are:
/// * mpsc
/// * spsc
/// * mpmc
pub fn connections(input: TokenStream) -> TokenStream
{
    todo!()
}
