use std::any::{Any, TypeId};
use std::marker::PhantomData;
use std::sync::Arc;

use dashmap::DashMap;
use tokio::sync::broadcast;
use tracing::debug;

use crate::error::{CoreError, CoreResult};

/// All events must implement this trait.
pub trait Event: Any + Send + Sync + Clone + 'static {}

/// Default broadcast channel capacity per event type.
const DEFAULT_CAPACITY: usize = 256;

struct Channel {
    sender: broadcast::Sender<Arc<dyn Any + Send + Sync>>,
}

impl Channel {
    fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }
}

/// High-performance, in-process pub/sub bus.
///
/// Each distinct event type gets its own broadcast channel, so a slow subscriber
/// on one event type cannot affect throughput on another.
pub struct EventBus {
    channels: DashMap<TypeId, Channel>,
    capacity: usize,
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(DEFAULT_CAPACITY)
    }
}

impl EventBus {
    pub fn new(per_type_capacity: usize) -> Self {
        Self { channels: DashMap::new(), capacity: per_type_capacity }
    }

    fn sender_for<E: Event>(&self) -> broadcast::Sender<Arc<dyn Any + Send + Sync>> {
        let type_id = TypeId::of::<E>();
        self.channels
            .entry(type_id)
            .or_insert_with(|| Channel::new(self.capacity))
            .sender
            .clone()
    }

    /// Publish an event to all current subscribers. Returns the number of receivers that got it.
    pub fn publish<E: Event>(&self, event: E) -> usize {
        let sender = self.sender_for::<E>();
        let arc: Arc<dyn Any + Send + Sync> = Arc::new(event);
        debug!(event_type = std::any::type_name::<E>(), "publishing event");
        sender.send(arc).unwrap_or(0)
    }

    /// Subscribe to an event type. Returns a typed receiver that yields cloned events.
    pub fn subscribe<E: Event>(&self) -> EventReceiver<E> {
        let receiver = self.sender_for::<E>().subscribe();
        EventReceiver { inner: receiver, _phantom: PhantomData }
    }

    /// Returns the number of active subscribers for a given event type.
    pub fn subscriber_count<E: Event>(&self) -> usize {
        let type_id = TypeId::of::<E>();
        self.channels
            .get(&type_id)
            .map(|ch| ch.sender.receiver_count())
            .unwrap_or(0)
    }
}

/// Typed receive handle returned by [`EventBus::subscribe`].
pub struct EventReceiver<E: Event> {
    inner: broadcast::Receiver<Arc<dyn Any + Send + Sync>>,
    _phantom: PhantomData<E>,
}

impl<E: Event> EventReceiver<E> {
    /// Await the next event of type `E`.
    pub async fn recv(&mut self) -> CoreResult<E> {
        loop {
            match self.inner.recv().await {
                Ok(arc) => {
                    if let Some(e) = arc.downcast_ref::<E>() {
                        return Ok(e.clone());
                    }
                }
                Err(broadcast::error::RecvError::Closed) => return Err(CoreError::BusClosed),
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!("event receiver lagged, skipped {n} messages");
                }
            }
        }
    }

    /// Try to receive without blocking.
    pub fn try_recv(&mut self) -> CoreResult<Option<E>> {
        loop {
            match self.inner.try_recv() {
                Ok(arc) => {
                    if let Some(e) = arc.downcast_ref::<E>() {
                        return Ok(Some(e.clone()));
                    }
                }
                Err(broadcast::error::TryRecvError::Empty) => return Ok(None),
                Err(broadcast::error::TryRecvError::Closed) => return Err(CoreError::BusClosed),
                Err(broadcast::error::TryRecvError::Lagged(n)) => {
                    tracing::warn!("event receiver lagged, skipped {n} messages");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    struct UserLoggedIn {
        username: String,
    }
    impl Event for UserLoggedIn {}

    #[derive(Clone, Debug, PartialEq)]
    struct OrderPlaced {
        order_id: u64,
    }
    impl Event for OrderPlaced {}

    #[tokio::test]
    async fn publish_before_subscribe_is_lost() {
        let bus = EventBus::default();
        // No subscribers yet — returns 0
        let n = bus.publish(UserLoggedIn { username: "alice".into() });
        assert_eq!(n, 0);
    }

    #[tokio::test]
    async fn subscriber_receives_event() {
        let bus = EventBus::default();
        let mut rx = bus.subscribe::<UserLoggedIn>();

        bus.publish(UserLoggedIn { username: "alice".into() });

        let evt = rx.recv().await.unwrap();
        assert_eq!(evt.username, "alice");
    }

    #[tokio::test]
    async fn multiple_subscribers_all_receive() {
        let bus = EventBus::default();
        let mut rx1 = bus.subscribe::<UserLoggedIn>();
        let mut rx2 = bus.subscribe::<UserLoggedIn>();

        let n = bus.publish(UserLoggedIn { username: "bob".into() });
        assert_eq!(n, 2);

        assert_eq!(rx1.recv().await.unwrap().username, "bob");
        assert_eq!(rx2.recv().await.unwrap().username, "bob");
    }

    #[tokio::test]
    async fn different_event_types_are_isolated() {
        let bus = EventBus::default();
        let mut login_rx = bus.subscribe::<UserLoggedIn>();
        let mut order_rx = bus.subscribe::<OrderPlaced>();

        bus.publish(OrderPlaced { order_id: 42 });
        bus.publish(UserLoggedIn { username: "carol".into() });

        assert_eq!(login_rx.recv().await.unwrap().username, "carol");
        assert_eq!(order_rx.recv().await.unwrap().order_id, 42);
    }

    #[tokio::test]
    async fn subscriber_count_tracks_receivers() {
        let bus = EventBus::default();
        assert_eq!(bus.subscriber_count::<UserLoggedIn>(), 0);

        let rx1 = bus.subscribe::<UserLoggedIn>();
        assert_eq!(bus.subscriber_count::<UserLoggedIn>(), 1);

        let rx2 = bus.subscribe::<UserLoggedIn>();
        assert_eq!(bus.subscriber_count::<UserLoggedIn>(), 2);

        drop(rx1);
        drop(rx2);
    }
}
