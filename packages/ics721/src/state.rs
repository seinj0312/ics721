use cosmwasm_schema::schemars::JsonSchema;
use cosmwasm_std::{Addr, Binary, ContractInfoResponse, Empty};
use cw_pause_once::PauseOrchestrator;
use cw_storage_plus::{Item, Map};
use serde::{Deserialize, Serialize};

use crate::token_types::{Class, ClassId, TokenId};

/// The code ID we will use for instantiating new cw721s.
pub const CW721_CODE_ID: Item<u64> = Item::new("a");
/// The proxy that this contract is receiving NFTs from, if any.
pub const PROXY: Item<Option<Addr>> = Item::new("b");
/// Manages contract pauses.
pub const PO: PauseOrchestrator = PauseOrchestrator::new("c", "d");

/// Maps classID (from NonFungibleTokenPacketData) to the cw721
/// contract we have instantiated for that classID.
pub const CLASS_ID_TO_NFT_CONTRACT: Map<ClassId, Addr> = Map::new("e");
/// Maps cw721 contracts to the classID they were instantiated for.
pub const NFT_CONTRACT_TO_CLASS_ID: Map<Addr, ClassId> = Map::new("f");

/// Maps between classIDs and classs. We need to keep this state
/// ourselves as cw721 contracts do not have class-level metadata.
pub const CLASS_ID_TO_CLASS: Map<ClassId, Class> = Map::new("g");

/// Maps (class ID, token ID) -> local channel ID. Used to determine
/// the local channel that NFTs have been sent out on.
pub const OUTGOING_CLASS_TOKEN_TO_CHANNEL: Map<(ClassId, TokenId), String> = Map::new("h");
/// Same as above, but for NFTs arriving at this contract.
pub const INCOMING_CLASS_TOKEN_TO_CHANNEL: Map<(ClassId, TokenId), String> = Map::new("i");
/// Maps (class ID, token ID) -> token metadata. Used to store
/// on-chain metadata for tokens that have arrived from other
/// chains. When a token arrives, it's metadata (regardless of if it
/// is `None`) is stored in this map. When the token is returned to
/// it's source chain, the metadata is removed from the map.
pub const TOKEN_METADATA: Map<(ClassId, TokenId), Option<Binary>> = Map::new("j");

#[derive(Deserialize)]
pub struct UniversalAllNftInfoResponse {
    pub access: UniversalOwnerOfResponse,
    pub info: UniversalNftInfoResponse,
}

#[derive(Deserialize)]
pub struct UniversalNftInfoResponse {
    pub token_uri: Option<String>,

    #[serde(skip_deserializing)]
    #[allow(dead_code)]
    extension: Empty,
}

/// Collection data send by ICS721 on source chain. It is an optional class data for interchain transfer to target chain.
/// ICS721 on target chain is free to use this data or not. Lik in case of `sg721-base` it uses owner for defining creator in collection info.
/// `ics721-base` uses name and symbol for instantiating new cw721 contract.
// NB: Please not cw_serde includes `deny_unknown_fields`: https://github.com/CosmWasm/cosmwasm/blob/v1.5.0/packages/schema-derive/src/cw_serde.rs
// For incoming data, parsing needs to be more lenient/less strict, so we use `serde` directly.
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug, PartialEq)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[schemars(crate = "cosmwasm_schema::schemars")]
#[serde(crate = "cosmwasm_schema::serde")]
pub struct CollectionData {
    pub owner: Option<String>,
    pub contract_info: Option<ContractInfoResponse>,
    pub name: String,
    pub symbol: String,
    pub num_tokens: Option<u64>,
}

#[derive(Deserialize)]
pub struct UniversalOwnerOfResponse {
    pub owner: String,

    #[serde(skip_deserializing)]
    #[allow(dead_code)]
    pub approvals: Vec<Empty>,
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{from_json, to_json_binary, Coin, Empty};

    use super::UniversalAllNftInfoResponse;

    #[test]
    fn test_universal_deserialize() {
        let start = cw721::AllNftInfoResponse::<Coin> {
            access: cw721::OwnerOfResponse {
                owner: "foo".to_string(),
                approvals: vec![],
            },
            info: cw721::NftInfoResponse {
                token_uri: None,
                extension: Coin::new(100, "ujuno"),
            },
        };
        let start = to_json_binary(&start).unwrap();
        let end: UniversalAllNftInfoResponse = from_json(start).unwrap();
        assert_eq!(end.access.owner, "foo".to_string());
        assert_eq!(end.access.approvals, vec![]);
        assert_eq!(end.info.token_uri, None);
        assert_eq!(end.info.extension, Empty::default())
    }
}
