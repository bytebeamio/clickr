use std::mem;

use tokio::time::{Duration, Instant};

use crate::client::Client;
use crate::{insert::Insert, Error};
use bytes::Bytes;

const DEFAULT_MAX_ENTRIES: u64 = 250_000;
const DEFAULT_MAX_DURATION: Duration = Duration::from_secs(10);
const MAX_TIME_BIAS: f64 = 0.10; // % of `max_duration`

pub struct Inserter {
    columns: Vec<String>,
    client: Client,
    table: String,
    max_entries: u64,
    max_duration: Duration,
    insert: Insert,
    next_insert_at: Instant,
    committed: Quantities,
    uncommitted_entries: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Quantities {
    pub entries: u64,
    pub transactions: u64,
}

impl Quantities {
    pub const ZERO: Quantities = Quantities {
        entries: 0,
        transactions: 0,
    };
}

impl Inserter {
    pub(crate) fn new(client: &Client, table: &str, columns: Vec<String>) -> Result<Self, Error> {
        Ok(Self {
            columns: columns.clone(),
            client: client.clone(),
            table: table.into(),
            max_entries: DEFAULT_MAX_ENTRIES,
            max_duration: DEFAULT_MAX_DURATION,
            insert: client.insert(table, columns)?,
            next_insert_at: Instant::now() + DEFAULT_MAX_DURATION,
            committed: Quantities::ZERO,
            uncommitted_entries: 0,
        })
    }

    pub fn with_max_entries(mut self, threshold: u64) -> Self {
        self.set_max_entries(threshold);
        self
    }

    pub fn with_max_duration(mut self, threshold: Duration) -> Self {
        self.set_max_duration(threshold);
        self
    }

    pub fn set_max_entries(&mut self, threshold: u64) {
        self.max_entries = threshold;
    }

    pub fn set_max_duration(&mut self, threshold: Duration) {
        let prev_insert_at = self
            .next_insert_at
            .checked_sub(self.max_duration)
            .unwrap_or_else(Instant::now);
        self.next_insert_at = prev_insert_at + threshold;
        self.max_duration = threshold;
    }

    #[inline]
    pub async fn write_bytes(&mut self, payload: Bytes) -> Result<(), Error> {
        self.uncommitted_entries += 1;
        self.insert.write_bytes(payload).await
    }

    #[inline]
    pub async fn write_slice(&mut self, payload: &[u8]) -> Result<(), Error> {
        self.uncommitted_entries += 1;
        self.insert.write_slice(payload).await
    }

    pub async fn commit(&mut self) -> Result<Quantities, Error> {
        if self.uncommitted_entries > 0 {
            self.committed.entries += self.uncommitted_entries;
            self.committed.transactions += 1;
            self.uncommitted_entries = 0;
        }

        let now = Instant::now();

        Ok(if self.is_threshold_reached(now) {
            self.next_insert_at = shifted_next_time(now, self.next_insert_at, self.max_duration);
            let new_insert = self.client.insert(&self.table, self.columns.clone())?; // Actually it mustn't fail.
            let insert = mem::replace(&mut self.insert, new_insert);
            insert.end().await?;
            mem::replace(&mut self.committed, Quantities::ZERO)
        } else {
            Quantities::ZERO
        })
    }

    pub async fn end(self) -> Result<Quantities, Error> {
        self.insert.end().await?;
        Ok(self.committed)
    }

    fn is_threshold_reached(&self, now: Instant) -> bool {
        self.committed.entries >= self.max_entries || now >= self.next_insert_at
    }
}

fn shifted_next_time(now: Instant, prev: Instant, max_duration: Duration) -> Instant {
    const MAX_TIME_BIAS_255: u32 = (MAX_TIME_BIAS * 255. + 0.5) as u32;

    let coef = (now.max(prev) - now.min(prev)).subsec_nanos() & 0xff;
    let max_bias = max_duration * MAX_TIME_BIAS_255 / 255;
    let bias = max_bias * coef / 255;

    prev + max_duration + 2 * bias - max_bias
}
