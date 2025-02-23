// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! An actor implementation to handle a user application runtime.

mod handlers;
mod requests;

use self::handlers::RequestHandler;
pub use self::requests::{BaseRequest, ContractRequest, ServiceRequest};
use crate::{ExecutionError, WasmExecutionError};
use futures::{
    channel::mpsc,
    select,
    stream::{FuturesUnordered, StreamExt, TryStreamExt},
};

/// A handler of application system APIs that runs as a separate actor.
///
/// Receives `Request`s from the application and handles them using the `Runtime`.
pub struct RuntimeActor<Runtime, Request> {
    runtime: Runtime,
    requests: mpsc::UnboundedReceiver<Request>,
}

impl<Runtime, Request> RuntimeActor<Runtime, Request>
where
    Runtime: RequestHandler<Request>,
    Request: std::fmt::Debug,
{
    /// Creates a new [`RuntimeActor`] using the provided `Runtime` to handle `Request`s.
    ///
    /// Returns the new [`RuntimeActor`] so that it can be executed later with the
    /// [`RuntimeActor::run`] method and the endpoint to send `Request`s to the actor.
    pub fn new(runtime: Runtime) -> (Self, mpsc::UnboundedSender<Request>) {
        let (sender, receiver) = mpsc::unbounded();

        let actor = RuntimeActor {
            runtime,
            requests: receiver,
        };

        (actor, sender)
    }

    /// Runs the [`RuntimeActor`], handling `Request`s until all the sender endpoints are closed.
    pub async fn run(mut self) -> Result<(), ExecutionError> {
        let mut active_requests = FuturesUnordered::new();

        // Run active request handlers concurrently with the incoming stream of `requests`
        loop {
            select! {
                maybe_result = active_requests.next() => {
                    if let Some(result) = maybe_result {
                        // A handler for a request just finished; check the result and return an
                        // error if there is one
                        result?;
                    }
                }
                maybe_request = self.requests.next() => match maybe_request {
                    Some(request) => {
                        // New request received, start handling it and add it to `active_requests`
                        active_requests.push(self.runtime.handle_request(request));
                    }
                    None => {
                        // All request sender endpoints have been dropped; leave the loop
                        break;
                    }
                },
            }
        }

        // Wait for all remaining active requests being handled, returning any errors as soon as
        // they appear
        active_requests.try_collect::<()>().await?;

        Ok(())
    }
}

/// Extension trait to help with sending requests to [`RuntimeActor`]s.
///
/// Prepares a channel for the actor to send a response back to the sender of the request.
pub trait SendRequestExt<Request> {
    /// Sends a request built by `builder`, returning a [`oneshot::Receiver`] for receiving the
    /// `Response`.
    fn send_request<Response>(
        &self,
        builder: impl FnOnce(oneshot::Sender<Response>) -> Request,
    ) -> Result<oneshot::Receiver<Response>, WasmExecutionError>
    where
        Response: Send;
}

impl<Request> SendRequestExt<Request> for mpsc::UnboundedSender<Request>
where
    Request: Send,
{
    fn send_request<Response>(
        &self,
        builder: impl FnOnce(oneshot::Sender<Response>) -> Request,
    ) -> Result<oneshot::Receiver<Response>, WasmExecutionError>
    where
        Response: Send,
    {
        let (response_sender, response_receiver) = oneshot::channel();
        let request = builder(response_sender);

        self.unbounded_send(request).map_err(|send_error| {
            assert!(
                send_error.is_disconnected(),
                "`send_request` should only be used with unbounded senders"
            );
            WasmExecutionError::MissingRuntimeResponse
        })?;

        Ok(response_receiver)
    }
}

#[cfg(test)]
mod tests;
