use std::cmp::min;
use crate::InferResponse;
use crate::{GenerateParameters, GenerateRequest};
use std::collections::VecDeque;
use nohash_hasher::IntMap;
use tokio::sync::mpsc::Receiver;
use text_generation_client::{
    Batch, ClientError, NextTokenChooserParameters, Request, StoppingCriteriaParameters,
};
use tokio::sync::oneshot::Sender;
use tokio::time::Instant;

/// In-flight request record
#[derive(Debug)]
pub(crate) struct Entry {
    /// Request
    pub request: GenerateRequest,
    /// Response sender to communicate between the Batcher and the batching_task
    pub response_tx: Sender<Result<InferResponse, ClientError>>,
    /// Number of tokens in the input
    pub input_length: usize,
    /// Instant when this entry was created
    pub time: Instant,
    /// Instant when this entry was added to a batch
    pub batch_time: Option<Instant>,
}

/// Request Queue
#[derive(Debug)]
pub(crate) struct Queue {
    receiver: Receiver<Entry>,
    buffer: VecDeque<Entry>,
    /// Id of the next entry
    next_id: u64,
    /// Id of the next batch
    next_batch_id: u64,
}


impl Queue {
    pub(crate) fn new(receiver: Receiver<Entry>) -> Self {
        Self { receiver, buffer: VecDeque::new(), next_id: 0, next_batch_id: 0 }
    }

    /// Get the next batch, blocking until available
    /// Corresponding entries are added to the entries map
    pub(crate) async fn next_batch(
        &mut self,
        max_size: usize,
        entries: &mut IntMap<u64, Entry>,
    ) -> Option<Batch> {
        loop {
            if self.buffer.is_empty() {
                match self.receiver.recv().await {
                    Some(ent) => self.buffer.push_back(ent),
                    None => return None,
                }
            }
            if let Some(batch) = self.try_next_batch(1, max_size, entries) {
                return Some(batch)
            }
        }
    }

    /// Get the next batch without blocking
    /// Corresponding entries are added to the entries map
    pub(crate) fn try_next_batch(
        &mut self,
        min_size: usize,
        max_size: usize,
        entries: &mut IntMap<u64, Entry>,
    ) -> Option<Batch> {
        while self.buffer.len() < max_size {
            match self.receiver.try_recv() {
                Ok(ent) => self.buffer.push_back(ent),
                _ => break,
            }
        }

        let len = self.buffer.len();
        if len < min_size || len == 0 {
            // Can't get minimum
            return None;
        }

        let now = Some(Instant::now());
        let requests = self.buffer.drain(..min(len, max_size))
            .map(|mut entry| {
                let id = self.next_id;
                self.next_id += 1;
                let request = Request {
                    id,
                    inputs: entry.request.inputs.clone(),
                    input_length: entry.input_length as u32,
                    parameters: Some((&entry.request.parameters).into()),
                    stopping_parameters: Some(entry.request.parameters.clone().into()),
                };
                entry.batch_time = now;
                entries.insert(id, entry);
                request
            })
            .collect::<Vec<Request>>();

        // Batch size
        let size = requests.len();
        let batch = Batch {
            id: self.next_batch_id,
            requests,
            size: size as u32,
        };
        // Increment batch id
        self.next_batch_id += 1;

        Some(batch)
    }
}


impl From<&GenerateParameters> for NextTokenChooserParameters {
    fn from(parameters: &GenerateParameters) -> Self {
        Self {
            temperature: parameters.temperature,
            top_k: parameters.top_k as u32,
            top_p: parameters.top_p,
            do_sample: parameters.do_sample,
        }
    }
}

impl From<GenerateParameters> for StoppingCriteriaParameters {
    fn from(parameters: GenerateParameters) -> Self {
        Self {
            stop_sequences: parameters.stop,
            max_new_tokens: parameters.max_new_tokens,
        }
    }
}
