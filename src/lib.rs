use game_spec::GameSpec;
use sp_core::crypto::{ExposeSecret, SecretString, Ss58Codec};
use sp_runtime::{
  self,
  traits::{IdentifyAccount, Verify},
  MultiSigner,
};
use std::{
  fs::File,
  io::BufReader,
  path::{Path, PathBuf},
};
use subxt::{tx::PairSigner, OnlineClient, PolkadotConfig};

use crate::game_spec::GameSpecBuilder;

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

type ResultOf<T> = Result<T, Box<dyn std::error::Error>>;
type FinalBiomeConfig = PolkadotConfig;
type Client = OnlineClient<FinalBiomeConfig>;
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
  let api = Client::from_url(endpoint).await?;

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
  endpoint: String,
  game_spec_path: PathBuf,
  organization_seed: String,
) -> ResultOf<()> {
  // init api client
  let api = Client::from_url(endpoint).await?;
  // load game spec from file
  let game_spec = load_game_spec(&game_spec_path)?;
  // construst the signer
  let pair = pair_from_suri::<sp_core::sr25519::Pair>(&organization_seed, None)?;
  let signer = PairSigner::new(pair.clone());
  // create game in the network
  post_to_node::<FinalBiomeConfig, sp_core::sr25519::Pair>(&api, game_spec, signer).await?;

  println!("Game spec has been imported to the network");
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

/// Creates an appropriate game configuration in the network
async fn post_to_node<Config, Pair>(
  api: &OnlineClient<Config>,
  game_spec: GameSpec,
  signer: PairSigner<Config, Pair>,
) -> ResultOf<()>
where
  Config: subxt::Config,
  Config::Signature: From<Pair::Signature>,
  Pair: sp_core::Pair,
  Pair::Public: Into<MultiSigner>,
  <Config::Signature as Verify>::Signer:
    From<Pair::Public> + IdentifyAccount<AccountId = Config::AccountId>,
  <<Config as subxt::Config>::ExtrinsicParams as subxt::tx::ExtrinsicParams<
    <Config as subxt::Config>::Index,
    <Config as subxt::Config>::Hash,
  >>::OtherParams: std::default::Default,
  <Config as subxt::Config>::Address: std::convert::From<<Config as subxt::Config>::AccountId>,
{
  // todo: make transactional creation of the configuration in the network

  let org_name = game_spec.organization_details.name.0;

  // 1. Create organization
  let payload = finalbiome::tx()
    .organization_identity()
    .create_organization(org_name);

  let _org_create = api
    .tx()
    .sign_and_submit_then_watch_default(&payload, &signer)
    .await?
    .wait_for_finalized_success()
    .await?;

  Ok(())
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

/// Try to parse given `uri` and print relevant information.
///
/// 1. Try to construct the `Pair` while using `uri` as input for [`sp_core::Pair::from_phrase`].
///
/// 2. Try to construct the `Pair` while using `uri` as input for
/// [`sp_core::Pair::from_string_with_seed`].
fn pair_from_suri<Pair>(suri: &str, password: Option<SecretString>) -> ResultOf<Pair>
where
  Pair: sp_core::Pair,
  Pair::Public: Into<MultiSigner>,
{
  let password = password.as_ref().map(|s| s.expose_secret().as_str());

  if let Ok((pair, _seed)) = Pair::from_phrase(suri, password) {
    Ok(pair)
  } else if let Ok((pair, _seed)) = Pair::from_string_with_seed(suri, password) {
    Ok(pair)
  } else {
    Err("Invalid phrase/URI given for the organization seed".into())
  }
}

/// Load the game spec form file by given path
fn load_game_spec<P>(path: P) -> ResultOf<GameSpec>
where
  P: AsRef<Path>,
{
  // check file exists
  let f = File::open(&path)?;
  let reader = BufReader::new(f);
  let game_spec: GameSpec = serde_json::from_reader(reader)?;
  Ok(game_spec)
}
