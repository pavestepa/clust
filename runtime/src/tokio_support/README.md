# tokio_support

Spawn Send futures on the Tokio scheduler. A task handle aborts on drop. Channels have bounded capacity, actors drain their mailboxes in order and mutex updates hold the guard for one synchronous state transition.

Module entry: [tokio_support](../tokio_support.rs).
