// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

#![cfg(any(feature = "wasmer", feature = "wasmtime"))]

mod utils;

use self::utils::create_dummy_user_application_description;
use counter::CounterAbi;
use linera_base::{
    data_types::{Amount, BlockHeight},
    identifiers::{ChainDescription, ChainId},
};
use linera_execution::{
    policy::ResourceControlPolicy, ExecutionResult, ExecutionRuntimeContext, ExecutionStateView,
    Operation, OperationContext, Query, QueryContext, RawExecutionResult, ResourceTracker,
    Response, SystemExecutionState, TestExecutionRuntimeContext, WasmApplication, WasmRuntime,
};
use linera_views::{memory::MemoryContext, views::View};
use serde_json::json;
use std::sync::Arc;
use test_case::test_case;

/// Test if the "counter" example application in `linera-sdk` compiled to a Wasm module can be
/// called correctly and consume the expected amount of fuel.
///
/// To update the bytecode files, run `linera-execution/update_wasm_fixtures.sh`.
#[cfg_attr(feature = "wasmer", test_case(WasmRuntime::Wasmer, 30_463; "wasmer"))]
#[cfg_attr(feature = "wasmer", test_case(WasmRuntime::WasmerWithSanitizer, 30_844; "wasmer_with_sanitizer"))]
#[cfg_attr(feature = "wasmtime", test_case(WasmRuntime::Wasmtime, 30_844; "wasmtime"))]
#[cfg_attr(feature = "wasmtime", test_case(WasmRuntime::WasmtimeWithSanitizer, 30_844; "wasmtime_with_sanitizer"))]
#[test_log::test(tokio::test(flavor = "multi_thread"))]
async fn test_fuel_for_counter_wasm_application(
    wasm_runtime: WasmRuntime,
    expected_fuel: u64,
) -> anyhow::Result<()> {
    let state = SystemExecutionState {
        description: Some(ChainDescription::Root(0)),
        ..Default::default()
    };
    let mut view =
        ExecutionStateView::<MemoryContext<TestExecutionRuntimeContext>>::from_system_state(state)
            .await;
    let app_desc = create_dummy_user_application_description();
    let app_id = view
        .system
        .registry
        .register_application(app_desc.clone())
        .await?;

    let application = WasmApplication::from_files(
        "tests/fixtures/counter_contract.wasm",
        "tests/fixtures/counter_service.wasm",
        wasm_runtime,
    )
    .await?;
    view.context()
        .extra
        .user_applications()
        .insert(app_id, Arc::new(application));

    let app_id = app_id.with_abi::<CounterAbi>();

    let context = OperationContext {
        chain_id: ChainId::root(0),
        height: BlockHeight(0),
        index: 0,
        authenticated_signer: None,
        next_message_index: 0,
    };
    let increments = [2_u64, 9, 7, 1000];
    let policy = ResourceControlPolicy {
        fuel: Amount::from_atto(1),
        ..ResourceControlPolicy::default()
    };
    let mut tracker = ResourceTracker::default();
    let amount = Amount::from_tokens(1);
    *view.system.balance.get_mut() = amount;
    for increment in &increments {
        let result = view
            .execute_operation(
                &context,
                &Operation::user(app_id, increment).unwrap(),
                &policy,
                &mut tracker,
            )
            .await?;
        assert_eq!(
            result,
            vec![ExecutionResult::User(
                app_id.forget_abi(),
                RawExecutionResult::default()
            )]
        );
    }
    assert_eq!(tracker.used_fuel, expected_fuel);

    let context = QueryContext {
        chain_id: ChainId::root(0),
    };
    let expected_value = async_graphql::Response::new(
        async_graphql::Value::from_json(json!({"value" : increments.into_iter().sum::<u64>()}))
            .unwrap(),
    );
    let request = async_graphql::Request::new("query { value }");
    let Response::User(serialized_value) = view
        .query_application(&context, &Query::user(app_id, &request).unwrap())
        .await?
    else {
        panic!("unexpected response")
    };
    assert_eq!(
        serde_json::from_slice::<async_graphql::Response>(&serialized_value).unwrap(),
        expected_value
    );
    Ok(())
}
