#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{from_binary, to_binary, Addr, DepsMut, Env, IbcMsg, MessageInfo, Response};
use cw2::set_contract_version;
use cw721_ibc::Cw721ReceiveMsg;
use cw_utils::nonpayable;

use crate::error::ContractError;
use crate::ibc::Ics721Packet;
use crate::msg::{ExecuteMsg, InstantiateMsg, TransferMsg};
use crate::state::{Config, CHANNEL_INFO, CONFIG};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:sg721-ics721";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
#[path = "contract_test.rs"]
mod contract_test;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let cfg = Config {
        default_timeout: msg.default_timeout,
    };
    CONFIG.save(deps.storage, &cfg)?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(msg) => execute_receive(deps, env, info, msg),
        ExecuteMsg::Transfer(msg) => execute_transfer(deps, env, msg, info.sender),
    }
}

pub fn execute_receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    wrapper: Cw721ReceiveMsg,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    let msg: TransferMsg = from_binary(&wrapper.msg)?;
    let api = deps.api;
    execute_transfer(deps, env, msg, api.addr_validate(&wrapper.sender)?)
}

pub fn execute_transfer(
    deps: DepsMut,
    env: Env,
    msg: TransferMsg,
    sender: Addr,
) -> Result<Response, ContractError> {
    // ensure the requested channel is registered
    if !CHANNEL_INFO.has(deps.storage, &msg.channel) {
        return Err(ContractError::NoSuchChannel { id: msg.channel });
    };

    // delta from user is in seconds
    let timeout_delta = match msg.timeout {
        Some(t) => t,
        None => CONFIG.load(deps.storage)?.default_timeout,
    };
    // timeout is in nanoseconds
    let timeout = env.block.time.plus_nanos(timeout_delta);

    // build ics721 packet
    let packet = Ics721Packet::new(
        &msg.class_id,
        None,
        msg.token_ids
            .iter()
            .map(|s| s.as_ref())
            .collect::<Vec<&str>>(),
        msg.token_uris
            .iter()
            .map(|s| s.as_ref())
            .collect::<Vec<&str>>(),
        sender.as_ref(),
        &msg.remote_address,
    );
    packet.validate()?;

    let msg = IbcMsg::SendPacket {
        channel_id: msg.channel,
        data: to_binary(&packet)?,
        timeout: timeout.into(),
    };

    // Note: we update local state when we get ack - do not count this transfer towards anything until acked
    // similar event messages like ibctransfer module

    // send response
    let res = Response::new()
        .add_message(msg)
        .add_attribute("action", "transfer")
        .add_attribute("sender", &packet.sender)
        .add_attribute("receiver", &packet.receiver)
        .add_attribute("class_id", &packet.class_id)
        .add_attribute("token_ids", &packet.token_ids.join(","));
    Ok(res)
}
