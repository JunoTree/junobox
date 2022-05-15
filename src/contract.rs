use crate::error::{ContractError};
use crate::msg::{
    ExecuteMsg, InstantiateMsg, BoxResponse,
    BoxCountResponse, QueryMsg, BoxMsg,
};
use crate::state::{State, FundsBox, BOXES, BOXES_COUNT, STATE};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coins, to_binary, BankMsg, Binary, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult, Uint128, Uint64,
};
use cw2::set_contract_version;
use cw_utils::must_pay;
use sha2::{Sha256, Digest};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw-junobox";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        denom: msg.denom,
        owner: info.sender.clone(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    BOXES_COUNT.save(deps.storage, &0)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateBoxes { boxes } => create_boxes(deps, info, boxes),
        ExecuteMsg::OpenBox { box_id, password } => open_box(deps, info, box_id, password),
    }
}

pub fn create_boxes(deps: DepsMut, info: MessageInfo, boxes: Vec<BoxMsg>) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;

    let mut funds_sum: Uint128 = Uint128::from(0u128);
    for box_item in &boxes {
        funds_sum += box_item.funds;
    }

    let payment = must_pay(&info, &state.denom)?;
    if payment < funds_sum {
        return Err(ContractError::InsufficientFunds {
            got: payment.u128(),
            needed: funds_sum.u128(),
        });
    }

    let mut ids = Vec::new();
    for box_item in boxes {
        let id = BOXES_COUNT.update(deps.storage, |count| -> Result<_, ContractError> {
            Ok(count + 1)
        })?;

        ids.push(id);

        BOXES.save(
            deps.storage,
            id,
            &FundsBox {
                creator: info.sender.clone(),
                funds: box_item.funds,
                hashed_password: box_item.hashed_password,
                opener: None,
            },
        )?;
    }

    let joined_ids: Vec<String> = ids
        .iter()
        .map(|n| n.to_string())
        .collect();

    let joined_ids_str = joined_ids.join(",");

    Ok(Response::new()
        .add_attribute("method", "create_boxes")
        .add_attribute("box_ids", joined_ids_str))
}

// Receive funds from box
pub fn open_box(deps: DepsMut, info: MessageInfo, box_id: u64, password: String) -> Result<Response, ContractError> {
    let found_box = BOXES.load(deps.storage, box_id)?;
    let state = STATE.load(deps.storage)?;

    let mut hasher = Sha256::new();
    hasher.update(&password);
    let hased_u8_vec = &hasher.finalize();
    let hashed_password = format!("{:x}", hased_u8_vec);
    
    if found_box.hashed_password != hashed_password {
        return Err(ContractError::IncorrectPassword {});
    }

    BOXES.save(
        deps.storage,
        box_id,
        &FundsBox {
            creator: found_box.creator,
            funds: found_box.funds.clone(),
            hashed_password: found_box.hashed_password,
            opener: Some(info.sender.clone()),
        },
    )?;

    Ok(Response::new().add_attribute("method", "open_box")
        .add_message(BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: coins(found_box.funds.u128(), state.denom),
        }))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::BoxCount {} => to_binary(&box_count(deps)?),
        QueryMsg::Box { box_id } => to_binary(&get_box(deps, box_id)?),
    }
}

fn box_count(deps: Deps) -> StdResult<BoxCountResponse> {
    let count = Uint64::from(BOXES_COUNT.load(deps.storage)?);
    Ok(BoxCountResponse { count })
}


fn get_box(deps: Deps, box_id: u64) -> StdResult<BoxResponse> {
    let funds_box = BOXES.load(deps.storage, box_id)?;
    Ok(BoxResponse { funds_box })
}
