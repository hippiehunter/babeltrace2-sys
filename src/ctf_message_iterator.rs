use crate::common_pipeline::CommonPipeline;
use crate::{BtResult, CtfPluginSourceFsInitParams, LoggingLevel, Message, NextStatus, RunStatus, StreamProperties, TraceProperties};
use std::collections::{BTreeSet, VecDeque};

pub struct CtfMessageIterator {
    pipeline: CommonPipeline,
    last_run_status: RunStatus,
    messages: VecDeque<Message>
}

impl CtfMessageIterator {
    pub fn new(log_level: LoggingLevel, params: &CtfPluginSourceFsInitParams) -> BtResult<Self> {
        let mut pipeline = CommonPipeline::new(log_level, params)?;

        // Do an initial run of the graph to connect and initialize all the components.
        // We'll have trace/stream metadata properties loaded and possibly some
        // events afterwards
        let last_run_status = pipeline.graph.run_once()?;

        Ok(CtfMessageIterator {
            pipeline,
            last_run_status,
            messages: VecDeque::<Message>::new()
        })
    }

    fn refill_events(&mut self) {
        // Example logic for refilling events
        // This is a placeholder; you'll replace it with your actual logic
        if self.messages.is_empty() {
            // Here, you would fetch new messages and update `self.messages`
            // For demonstration, let's assume we call a method that fills `self.messages`
            // with new messages if `last_run_status` indicates more messages are available.
            if self.last_run_status == RunStatus::Ok || self.last_run_status == RunStatus::TryAgain {
                if let Some(msg_itr) = self.pipeline.proxy_state.as_mut().msg_iter.as_mut() {
                    if let Ok((status, message_array)) = msg_itr.next_message_array() {
                        match status {
                            NextStatus::Ok => {
                                self.last_run_status = RunStatus::Ok;
                                self.messages.extend(message_array.as_slice().iter().map(|m| Message::from_raw(*m)))
                            },
                            NextStatus::TryAgain => {
                                self.last_run_status = RunStatus::TryAgain
                            },
                            NextStatus::End => {
                                self.last_run_status = RunStatus::End
                            }
                        }
                    }
                } else {
                    self.last_run_status = RunStatus::End;
                }
            } else {
                self.last_run_status = RunStatus::End;
            }
        }
    }

    pub fn trace_properties(&self) -> &TraceProperties {
        &self.pipeline.proxy_state.as_ref().trace_properties
    }

    pub fn stream_properties(&self) -> &BTreeSet<StreamProperties> {
        &self.pipeline.proxy_state.as_ref().stream_properties
    }
}

impl Iterator for CtfMessageIterator {
    type Item = Message;

    fn next(&mut self) -> Option<Self::Item> {
        if self.last_run_status == RunStatus::Ok || self.last_run_status == RunStatus::TryAgain {
            if self.messages.is_empty() {
                self.refill_events();
            }
        }
        // Attempt to pop a message from the front of the list
        if !self.messages.is_empty() {
            self.messages.pop_front()
        } else {
            None
        }
    }
}
