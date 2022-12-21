use sp_core::crypto::Ss58Codec;
use sp_runtime::{self, MultiSigner};
use std::{fs::File, path::PathBuf};
use subxt::{OnlineClient, SubstrateConfig};

use crate::game_spec::GameSpecBuilder;

type ResultOf<T> = Result<T, Box<dyn std::error::Error>>;

#[subxt::subxt(
  runtime_metadata_path = "artifacts/finalbiome_metadata.scale",
  derive_for_all_types = "Clone, PartialEq, Eq",
  derive_for_type(
    type = "pallet_organization_identity::types::OrganizationDetails",
    derive = "serde::Serialize, serde::Deserialize"
  ),
  derive_for_type(
    type = "sp_runtime::bounded::bounded_vec::BoundedVec",
    derive = "serde::Serialize, serde::Deserialize"
  ),
  derive_for_type(
    type = "pallet_organization_identity::types::AirDropAsset",
    derive = "serde::Serialize, serde::Deserialize"
  ),
  derive_for_type(
    type = "pallet_support::types::fungible_asset_id::FungibleAssetId",
    derive = "serde::Serialize, serde::Deserialize"
  ),
  derive_for_type(
    type = "pallet_support::types::fungible_asset_balance::FungibleAssetBalance",
    derive = "serde::Serialize, serde::Deserialize"
  ),
  derive_for_type(
    type = "pallet_support::types::non_fungible_class_id::NonFungibleClassId",
    derive = "serde::Serialize, serde::Deserialize"
  ),
  derive_for_type(
    type = "pallet_support::Attribute",
    derive = "serde::Serialize, serde::Deserialize"
  ),
  derive_for_type(
    type = "pallet_support::AttributeValue",
    derive = "serde::Serialize, serde::Deserialize"
  ),
  derive_for_type(
    type = "pallet_support::NumberAttribute",
    derive = "serde::Serialize, serde::Deserialize"
  )
)]
pub mod finalbiome {}

mod game_spec;

/// Export game spec to file.
///
/// The following items are exported:
/// - Organization Details
/// - Organization Members
/// - Users of orgamization
pub async fn export_game_spec(
  endpoint: String,
  organization: String,
  game_spec_path: PathBuf,
  overwrite_file: bool,
) -> ResultOf<()> {
  // init api client
  let api = OnlineClient::<SubstrateConfig>::from_url(endpoint).await?;

  let node_version = fetch_node_version(&api);
  let org_details = fetch_organization_details(&api, &organization);

  let game_spec_builder = GameSpecBuilder::new();
  let game_spec = game_spec_builder
    .version(node_version.await?)
    .organization_details(org_details.await?)
    .try_build()?;

  // save to file
  if !overwrite_file {
    // check file exists
    if File::open(&game_spec_path).is_ok() {
      return Err(format!("File {} already exists", game_spec_path.display()).into());
    }
  }
  let f = File::create(game_spec_path)?;
  serde_json::to_writer(f, &game_spec)?;

  println!("Game spec has been exported");

  Ok(())
}

/// Import game spec into the network.
pub async fn import_game_spec(
  _endpoint: String,
  _organization: String,
  _game_spec: PathBuf,
  _organization_seed: String,
) -> ResultOf<()> {
  Ok(())
}

/// Fetch version of the node
async fn fetch_node_version<Config>(api: &OnlineClient<Config>) -> ResultOf<String>
where
  Config: subxt::Config,
{
  api
    .rpc()
    .system_version()
    .await
    .map_err(|_| "Cannot fetch version of the node".into())
}

/// Fetch organization details by organization address
async fn fetch_organization_details<Config>(
  api: &OnlineClient<Config>,
  organization: &str,
) -> ResultOf<
  finalbiome::runtime_types::pallet_organization_identity::types::OrganizationDetails<
    finalbiome::runtime_types::sp_runtime::bounded::bounded_vec::BoundedVec<u8>,
  >,
>
where
  Config: subxt::Config,
{
  // get accountId by SS58 address
  let organization_id: sp_runtime::AccountId32 =
    public_from_uri::<sp_core::sr25519::Pair>(organization)?.into();
  let address = finalbiome::storage()
    .organization_identity()
    .organizations(organization_id);
  let organization_details = api.storage().fetch(&address, None).await?;
  organization_details
    .ok_or_else(|| format!("Organization by address {} not found", organization).into())
}

/// Transform uri str to Public key
fn public_from_uri<Pair>(uri: &str) -> ResultOf<Pair::Public>
where
  Pair: sp_core::Pair,
  Pair::Public: Into<MultiSigner>,
{
  if let Ok((public_key, _network)) = Pair::Public::from_string_with_version(uri) {
    Ok(public_key)
  } else {
    Err("Invalid organization address/URI given".into())
  }
}
