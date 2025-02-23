// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Conversions from types generated by `wit-bindgen`.
//!
//! Allows converting types returned from a Wasm module into types that can be used with the rest
//! of the crate.

#![allow(clippy::duplicate_mod)]

use super::{contract, contract_system_api, service_system_api};
use crate::{
    ApplicationCallResult, ChannelName, Destination, RawExecutionResult, RawOutgoingMessage,
    SessionCallResult, SessionId, UserApplicationId,
};
use linera_base::{
    crypto::CryptoHash,
    data_types::BlockHeight,
    identifiers::{BytecodeId, ChainId, MessageId},
};

impl From<contract::SessionCallResult> for (SessionCallResult, Vec<u8>) {
    fn from(result: contract::SessionCallResult) -> Self {
        let session_call_result = SessionCallResult {
            inner: result.inner.into(),
            close_session: result.new_state.is_some(),
        };

        let updated_session_state = result.new_state.unwrap_or_default();

        (session_call_result, updated_session_state)
    }
}

impl From<contract::ApplicationCallResult> for ApplicationCallResult {
    fn from(result: contract::ApplicationCallResult) -> Self {
        ApplicationCallResult {
            create_sessions: result.create_sessions,
            execution_result: result.execution_result.into(),
            value: result.value,
        }
    }
}

impl From<contract::OutgoingMessage> for RawOutgoingMessage<Vec<u8>> {
    fn from(message: contract::OutgoingMessage) -> Self {
        Self {
            destination: message.destination.into(),
            authenticated: message.authenticated,
            is_skippable: message.is_skippable,
            message: message.message,
        }
    }
}

impl From<contract::ExecutionResult> for RawExecutionResult<Vec<u8>> {
    fn from(result: contract::ExecutionResult) -> Self {
        let messages = result
            .messages
            .into_iter()
            .map(RawOutgoingMessage::from)
            .collect();

        let subscribe = result
            .subscribe
            .into_iter()
            .map(|(subscription, chain_id)| (subscription.into(), chain_id.into()))
            .collect();

        let unsubscribe = result
            .unsubscribe
            .into_iter()
            .map(|(subscription, chain_id)| (subscription.into(), chain_id.into()))
            .collect();

        RawExecutionResult {
            authenticated_signer: None,
            messages,
            subscribe,
            unsubscribe,
        }
    }
}

impl From<contract::Destination> for Destination {
    fn from(guest: contract::Destination) -> Self {
        match guest {
            contract::Destination::Recipient(chain_id) => Destination::Recipient(chain_id.into()),
            contract::Destination::Subscribers(subscription) => {
                Destination::Subscribers(subscription.into())
            }
        }
    }
}

impl From<contract::ChannelName> for ChannelName {
    fn from(guest: contract::ChannelName) -> Self {
        guest.name.into()
    }
}

impl From<contract::CryptoHash> for CryptoHash {
    fn from(guest: contract::CryptoHash) -> Self {
        let integers = [guest.part1, guest.part2, guest.part3, guest.part4];
        CryptoHash::from(integers)
    }
}

impl From<contract::ChainId> for ChainId {
    fn from(guest: contract::ChainId) -> Self {
        ChainId(guest.into())
    }
}

impl From<contract_system_api::SessionId> for SessionId {
    fn from(guest: contract_system_api::SessionId) -> Self {
        SessionId {
            application_id: guest.application_id.into(),
            index: guest.index,
        }
    }
}

impl From<contract_system_api::ApplicationId> for UserApplicationId {
    fn from(guest: contract_system_api::ApplicationId) -> Self {
        UserApplicationId {
            bytecode_id: guest.bytecode_id.into(),
            creation: guest.creation.into(),
        }
    }
}

impl From<contract_system_api::MessageId> for BytecodeId {
    fn from(guest: contract_system_api::MessageId) -> Self {
        BytecodeId::new(guest.into())
    }
}

impl From<contract_system_api::MessageId> for MessageId {
    fn from(guest: contract_system_api::MessageId) -> Self {
        MessageId {
            chain_id: guest.chain_id.into(),
            height: BlockHeight(guest.height),
            index: guest.index,
        }
    }
}

impl From<contract_system_api::CryptoHash> for ChainId {
    fn from(guest: contract_system_api::CryptoHash) -> Self {
        ChainId(guest.into())
    }
}

impl From<contract_system_api::CryptoHash> for CryptoHash {
    fn from(guest: contract_system_api::CryptoHash) -> Self {
        let integers = [guest.part1, guest.part2, guest.part3, guest.part4];
        CryptoHash::from(integers)
    }
}

impl From<service_system_api::ApplicationId> for UserApplicationId {
    fn from(guest: service_system_api::ApplicationId) -> Self {
        UserApplicationId {
            bytecode_id: guest.bytecode_id.into(),
            creation: guest.creation.into(),
        }
    }
}

impl From<service_system_api::MessageId> for BytecodeId {
    fn from(guest: service_system_api::MessageId) -> Self {
        BytecodeId::new(guest.into())
    }
}

impl From<service_system_api::MessageId> for MessageId {
    fn from(guest: service_system_api::MessageId) -> Self {
        MessageId {
            chain_id: guest.chain_id.into(),
            height: BlockHeight(guest.height),
            index: guest.index,
        }
    }
}

impl From<service_system_api::CryptoHash> for ChainId {
    fn from(guest: service_system_api::CryptoHash) -> Self {
        ChainId(guest.into())
    }
}

impl From<service_system_api::CryptoHash> for CryptoHash {
    fn from(guest: service_system_api::CryptoHash) -> Self {
        let integers = [guest.part1, guest.part2, guest.part3, guest.part4];
        CryptoHash::from(integers)
    }
}
