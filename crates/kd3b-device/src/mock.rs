use crate::PacketTransport;

/// Deterministic in-memory transport that records successful packet writes.
pub struct MockTransport {
    recorded_writes: Vec<Vec<u8>>,
    failure_index: Option<usize>,
    next_write_index: usize,
}

impl MockTransport {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            recorded_writes: Vec::new(),
            failure_index: None,
            next_write_index: 0,
        }
    }

    pub const fn fail_on_write(&mut self, write_index: usize) {
        self.failure_index = Some(write_index);
    }

    #[must_use]
    pub fn recorded_writes(&self) -> &[Vec<u8>] {
        &self.recorded_writes
    }
}

impl Default for MockTransport {
    fn default() -> Self {
        Self::new()
    }
}

/// Error returned for a configured mock write failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MockTransportError {
    write_index: usize,
}

impl MockTransportError {
    #[must_use]
    pub const fn write_index(&self) -> usize {
        self.write_index
    }
}

impl PacketTransport for MockTransport {
    type Error = MockTransportError;

    fn write_packet(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
        let write_index = self.next_write_index;
        self.next_write_index += 1;

        if self.failure_index == Some(write_index) {
            return Err(MockTransportError { write_index });
        }

        self.recorded_writes.push(bytes.to_vec());
        Ok(())
    }
}
