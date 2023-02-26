use proc_macro::TokenStream;
mod channels;
mod connections;

#[proc_macro]
/// Connections describes the information about the channels, such as what
/// channel connects to another channel. The type of channel that we want and
/// then also the buffer size on each channel.
///
/// The syntax for this is as follows:
///
/// `name, data; channel_type, buffer_size; []`
///
/// ```rust
/// connections! {
///     subscriptions, SubscriptionEnum; mpmc, 10;
///     incoming_http, HttpRequestsStruct; mpsc, 5; (NoisyChannel, SillySender, RandomReceiver)
/// }
/// ```
///The types of channels that are supported are:
/// * mpsc
/// * mpmc
///
/// Any Custom channel usage must implement the [`catacombs_types::MpscChannel`]
/// or the [`catacombs_types::MpmcChannel`] trait. and compatible with
/// the defined channel type.
pub fn connections(input: TokenStream) -> TokenStream
{
    connections::parse_connections(input.into()).into()
}

#[proc_macro_attribute]
/// The channel attribute is used to mark what channels we want to inject into
/// the given struct. The syntax for this is as follows:
/// ```rust
/// #[channel(subscriptions_rx, incoming_http_tx)]
/// struct MyStruct {
///   send_msgs: u16,
///   ...
/// }
///
/// impl MyStruct {
///     #[inject(subscriptions_tx, incoming_http_rx)]
///     pub fn new() -> Self {
///         Self { send_msgs: 0 }
///     }
///
///     pub async fn process_sub_data(&mut self) {
///         if let Ok(message) = self.subscriptions_rx.recv().await {
///             ...
///         }
///     }
///
///     pub async fn send_new_http(&mut self, request: HttpRequestsStruct) {
///         self.incoming_http_tx.send(request).await.unwrap();
///         self.send_msgs += 1;
///     }
/// }
/// ```
/// The channels the code is accessing above are the channels that were defined
/// in the [`connections`] macro. Here is the example we generated from
/// ```rust
/// connections! {
///     subscriptions, SubscriptionEnum; mpmc, 10;
///     incoming_http, HttpRequestsStruct; mpsc, 5; NoisyChannel
/// }
/// ```
pub fn channel(args: TokenStream, input: TokenStream) -> TokenStream
{
    channels::parse_struct(args.into(), input.into()).into()
}

#[proc_macro_attribute]
/// The channel attribute is used to mark what channels we want to inject into
/// the given struct. The syntax for this is as follows:
/// ```rust
/// #[channel(subscriptions_rx, incoming_http_tx)]
/// struct MyStruct {
///   send_msgs: u16,
///   ...
/// }
///
/// impl MyStruct {
///     #[inject(subscriptions_tx, incoming_http_rx)]
///     pub fn new() -> Self {
///         Self { send_msgs: 0 }
///     }
///
///     pub async fn process_sub_data(&mut self) {
///         if let Ok(message) = self.subscriptions_rx.recv().await {
///             ...
///         }
///     }
///
///     pub async fn send_new_http(&mut self, request: HttpRequestsStruct) {
///         self.incoming_http_tx.send(request).await.unwrap();
///         self.send_msgs += 1;
///     }
/// }
/// ```
/// The channels the code is accessing above are the channels that were defined
/// in the [`connections`] macro. Here is the example we generated from
/// ```rust
/// connections! {
///     subscriptions, SubscriptionEnum; mpmc, 10;
///     incoming_http, HttpRequestsStruct; mpsc, 5; NoisyChannel
/// }
/// ```
pub fn inject(args: TokenStream, input: TokenStream) -> TokenStream
{
    channels::parse_inject(args.into(), input.into()).into()
}
