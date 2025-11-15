#![allow(deprecated)]

use crux_core::capability::{CapabilityContext, Operation};
use serde::{Deserialize, Serialize};

// Operations that the Shell needs to perform for Centrifugo
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CentrifugoOperation {
    Subscribe { channel: String },
    Unsubscribe { channel: String },
    SubscribeAll,
    UnsubscribeAll,
}

// The output from Centrifugo operations (shell tells us what happened)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CentrifugoOutput {
    Subscribed { channel: String },
    Unsubscribed { channel: String },
    Message { channel: String, data: String },
    Error { message: String },
}

impl Operation for CentrifugoOperation {
    type Output = CentrifugoOutput;
}

// The Centrifugo capability
pub struct Centrifugo<Ev> {
    context: CapabilityContext<CentrifugoOperation, Ev>,
}

impl<Ev> Centrifugo<Ev>
where
    Ev: 'static,
{
    pub fn new(context: CapabilityContext<CentrifugoOperation, Ev>) -> Self {
        Self { context }
    }

    pub fn subscribe(&self, _channel: &str) {
        // Will be implemented when shell integration is ready
    }

    pub fn unsubscribe(&self, _channel: &str) {
        // Will be implemented when shell integration is ready
    }

    pub fn subscribe_all(&self) {
        // Will be implemented when shell integration is ready
    }

    pub fn unsubscribe_all(&self) {
        // Will be implemented when shell integration is ready
    }
}

impl<Ev> crux_core::Capability<Ev> for Centrifugo<Ev> {
    type Operation = CentrifugoOperation;
    type MappedSelf<MappedEv> = Centrifugo<MappedEv>;

    fn map_event<F, NewEv>(&self, f: F) -> Self::MappedSelf<NewEv>
    where
        F: Fn(NewEv) -> Ev + Send + Sync + 'static,
        Ev: 'static,
        NewEv: 'static + Send,
    {
        Centrifugo::new(self.context.map_event(f))
    }
}
