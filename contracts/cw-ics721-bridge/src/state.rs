use cosmwasm_std::{Addr, Empty};
use cw_storage_plus::{Item, Map};
use serde::Deserialize;

/// Maps channel IDs to the escrow address for that channel.
pub const CHANNELS: Map<String, Addr> = Map::new("channels");

/// The code ID we will use for instantiating new cw721s.
pub const CW721_ICS_CODE_ID: Item<u64> = Item::new("cw721_code_id");

// The code ID we will use when instantiating escrow contracts.
pub const ESCROW_CODE_ID: Item<u64> = Item::new("escrow_code_id");

/// Maps classID (from NonFungibleTokenPacketData) to the cw721
/// contract we have instantiated for that classID.
pub const CLASS_ID_TO_NFT_CONTRACT: Map<String, Addr> = Map::new("class_id_to_contract");
/// Maps cw721 contracts to the classID they were instantiated for.
pub const NFT_CONTRACT_TO_CLASS_ID: Map<Addr, String> = Map::new("contract_to_class_id");
/// Maps between classIDs and classUris. We need to keep this state
/// ourselves as cw721 contracts do not.
pub const CLASS_ID_TO_CLASS_URI: Map<String, Option<String>> = Map::new("class_id_to_class_uri");

#[derive(Deserialize)]
pub struct UniversalNftInfoResponse {
    pub token_uri: Option<String>,

    #[serde(skip_deserializing)]
    #[allow(dead_code)]
    extension: Empty,
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{from_binary, to_binary, Coin, Empty};

    use super::UniversalNftInfoResponse;

    #[test]
    fn test_universal_deserialize() {
        let start = cw721::NftInfoResponse::<Coin> {
            token_uri: None,
            extension: Coin::new(100, "ujuno"),
        };
        let start = to_binary(&start).unwrap();
        let end: UniversalNftInfoResponse = from_binary(&start).unwrap();
        assert_eq!(end.token_uri, None);
        assert_eq!(end.extension, Empty::default())
    }
}
