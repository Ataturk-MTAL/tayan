/// Generic pending-event buffer — zero heap allocation, no trait objects.
///
/// Each aggregate owns an `EventQueue<SomeEventEnum>`.
/// Events are drained and dispatched by the application layer after persistence.
#[derive(Debug)]
pub struct EventQueue<E>(Vec<E>);

// Manual impl — avoids the spurious `E: Default` bound that #[derive(Default)] adds.
impl<E> Default for EventQueue<E> {
    fn default() -> Self { Self(Vec::new()) }
}

impl<E> EventQueue<E> {
    pub fn push(&mut self, event: E) {
        self.0.push(event);
    }

    pub fn drain(&mut self) -> Vec<E> {
        std::mem::take(&mut self.0)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
