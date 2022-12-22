use game_spec::GameSpec;
use parity_scale_codec::Decode;
use sp_core::{
  crypto::{ExposeSecret, SecretString, Ss58Codec},
  hexdisplay::HexDisplay,
};
use sp_runtime::{
  self,
  traits::{IdentifyAccount, Verify},
  AccountId32, MultiAddress, MultiSigner,
};
use std::{
  collections::HashMap,
  fs::File,
  io::BufReader,
  path::{Path, PathBuf},
};
use subxt::{
  storage::address::{StorageHasher, StorageMapKey},
  tx::PairSigner,
  OnlineClient, PolkadotConfig,
};

use crate::{game_spec::GameSpecBuilder, utils::AllKeyIter};

#[subxt::subxt(
  runtime_metadata_path = "artifacts/finalbiome_metadata.scale",
  derive_for_all_types = "Clone, PartialEq, Eq",
  derive_for_type(
    type = "pallet_organization_identity::types::OrganizationDetails",
    derive = "serde::Serialize, serde::Deserialize"
  ),
  derive_for_type(
    type = "pallet_fungible_assets::types::AssetDetails",
    derive = "serde::Serialize, serde::Deserialize"
  ),
  derive_for_type(
    type = "pallet_fungible_assets::types::TopUppedFA",
    derive = "serde::Serialize, serde::Deserialize"
  ),
  derive_for_type(
    type = "pallet_fungible_assets::types::CupFA",
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
    derive = "serde::Serialize, serde::Deserialize, Debug, Hash"
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
mod utils;

type ResultOf<T> = Result<T, Box<dyn std::error::Error>>;
type FinalBiomeConfig = PolkadotConfig;
type Client = OnlineClient<FinalBiomeConfig>;
pub(crate) type FungibleAssetDetails =
  finalbiome::runtime_types::pallet_fungible_assets::types::AssetDetails<
    AccountId32,
    finalbiome::runtime_types::sp_runtime::bounded::bounded_vec::BoundedVec<u8>,
  >;
pub(crate) type FungibleAssetId =
  finalbiome::runtime_types::pallet_support::types::fungible_asset_id::FungibleAssetId;

pub(crate) type FungibleAssetIds = Vec<(FungibleAssetId, FungibleAssetDetails)>;
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
  // get current hash
  let block_hash = fetch_curr_hash(&api).await?;
  // set organization account id from SS58 address
  let organization_id: AccountId32 =
    public_from_uri::<sp_core::sr25519::Pair>(&organization)?.into();

  let node_version = fetch_node_version(&api);
  let org_details = fetch_organization_details(&api, &organization_id, block_hash);
  let org_members = fetch_organization_members(&api, &organization_id, block_hash);
  let fas = fetch_fas(&api, &organization_id, block_hash);

  let game_spec_builder = GameSpecBuilder::new();
  let game_spec = game_spec_builder
    .version(node_version.await?)
    .hash(format!("0x{}", HexDisplay::from(&block_hash.as_ref())))
    .organization_details(org_details.await?)
    .organization_members(org_members.await?)
    .fa(fas.await?)
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
  manager_seed: String,
) -> ResultOf<()> {
  // init api client
  let api = Client::from_url(endpoint).await?;
  // load game spec from file
  let game_spec = load_game_spec(&game_spec_path)?;
  // construst the game signer
  let organization_pair = pair_from_suri::<sp_core::sr25519::Pair>(&organization_seed, None)?;
  let organization_signer = PairSigner::new(organization_pair);
  // construst the manager signer
  let manager_pair = pair_from_suri::<sp_core::sr25519::Pair>(&manager_seed, None)?;
  let manager_signer = PairSigner::new(manager_pair.clone());
  // create game in the network
  post_to_node::<FinalBiomeConfig, sp_core::sr25519::Pair>(
    &api,
    game_spec,
    organization_signer,
    manager_signer,
  )
  .await?;

  println!("Game spec has been imported to the network");
  Ok(())
}

/// Fetch version of the node
async fn fetch_node_version<T>(api: &OnlineClient<T>) -> ResultOf<String>
where
  T: subxt::Config,
{
  api
    .rpc()
    .system_version()
    .await
    .map_err(|_| "Cannot fetch version of the node".into())
}

// Fetch a concrete block hash to export from. We do this so that if new blocks
// are produced midway through export, we continue to export at the block
// we started with and not the new block.
async fn fetch_curr_hash<T>(api: &OnlineClient<T>) -> ResultOf<T::Hash>
where
  T: subxt::Config,
{
  api
    .rpc()
    .block_hash(None)
    .await?
    .ok_or_else(|| "Cannot fetch current hash".into())
}

/// Fetch organization details by organization address
async fn fetch_organization_details<T>(
  api: &OnlineClient<T>,
  organization_id: &AccountId32,
  block_hash: T::Hash,
) -> ResultOf<
  finalbiome::runtime_types::pallet_organization_identity::types::OrganizationDetails<
    finalbiome::runtime_types::sp_runtime::bounded::bounded_vec::BoundedVec<u8>,
  >,
>
where
  T: subxt::Config,
{
  let address = finalbiome::storage()
    .organization_identity()
    .organizations(organization_id);
  let organization_details = api.storage().fetch(&address, Some(block_hash)).await?;
  organization_details.ok_or_else(|| "Organization not found".into())
}

async fn fetch_organization_members<T>(
  api: &OnlineClient<T>,
  organization_id: &AccountId32,
  block_hash: T::Hash,
) -> ResultOf<Vec<sp_runtime::AccountId32>>
where
  T: subxt::Config,
{
  // Iterate over the membersOf storage to get all managers of the game
  let key_addr = finalbiome::storage()
    .organization_identity()
    .members_of(organization_id, organization_id);
  // Obtain the root bytes
  let mut query_key = key_addr.to_root_bytes();
  // We know that the first key is a AccountId32 and is hashed by Blake2_128Concat.
  // We can build a `StorageMapKey` that replicates that, and append those bytes to the above.
  StorageMapKey::new(organization_id, StorageHasher::Blake2_128Concat).to_bytes(&mut query_key);

  // Iterate over keys at that address and collect values to vec
  let mut members = vec![];
  let mut iter = AllKeyIter::new(api, query_key, block_hash, 10);
  while let Some(key) = iter.next().await? {
    // we need last 32 bytes - the member id.
    let member: [u8; 32] = key.0.as_slice()[key.0.len() - 32..]
      .try_into()
      .expect("we get last 32 bytes but it is not");
    let member_id: sp_runtime::AccountId32 = sp_runtime::AccountId32::new(member);
    println!(
      "Member of {} is {}",
      &organization_id.to_ss58check(),
      &member_id.to_ss58check()
    );
    members.push(member_id);
  }
  Ok(members)
}

async fn fetch_fas<T>(
  api: &OnlineClient<T>,
  organization_id: &AccountId32,
  block_hash: T::Hash,
) -> ResultOf<Vec<(FungibleAssetId, FungibleAssetDetails)>>
where
  T: subxt::Config,
{
  // 1. Fetch the fa ids belonging to the game
  let mut asset_ids = vec![];
  // Iterate over the membersOf storage to get all managers of the game
  let key_addr = finalbiome::storage().fungible_assets().assets_of_root();
  // Obtain the root bytes
  let mut query_key = key_addr.to_root_bytes();
  // We know that the first key is a AccountId32 and is hashed by Blake2_128Concat.
  // We can build a `StorageMapKey` that replicates that, and append those bytes to the above.
  StorageMapKey::new(organization_id, StorageHasher::Blake2_128Concat).to_bytes(&mut query_key);
  // Iterate over keys at that address and collect values to vec
  let mut iter = AllKeyIter::new(api, query_key, block_hash, 10);
  while let Some(key) = iter.next().await? {
    // we need last 4 bytes (u32) - the asset id.
    let asset_id_encoded = key.0.as_slice()[key.0.len() - 4..].to_vec();
    let asset_id =
      finalbiome::runtime_types::pallet_support::types::fungible_asset_id::FungibleAssetId::decode(
        &mut &*asset_id_encoded,
      )?;

    // println!("Member of {} is {}", &organization_id.to_ss58check(), &member_id.to_ss58check());
    asset_ids.push(asset_id);
  }

  // 2. Fetch details about each found assets
  let mut fa_details = vec![];
  for asset_id in asset_ids {
    let address = finalbiome::storage().fungible_assets().assets(&asset_id);
    let details: FungibleAssetDetails = api
      .storage()
      .fetch(&address, Some(block_hash))
      .await?
      .ok_or_else(|| format!("FA {:?} not found", asset_id))?;

    fa_details.push((asset_id, details));
  }

  Ok(fa_details)
}

/// Creates an appropriate game configuration in the network
async fn post_to_node<Config, Pair>(
  api: &OnlineClient<Config>,
  game_spec: GameSpec,
  organization_signer: PairSigner<Config, Pair>,
  manager_signer: PairSigner<Config, Pair>,
) -> ResultOf<()>
where
  Config: subxt::Config<AccountId = AccountId32>,
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
    .sign_and_submit_then_watch_default(&payload, &organization_signer)
    .await?
    .wait_for_in_block()
    .await?
    .wait_for_success()
    .await?;

  // 2. Add members
  // Also add the manager that is explicitly passed to the app (if the manager is not included in
  // the specification)
  let expl_manager = manager_signer.account_id().clone();
  let mut members = game_spec.organization_members.clone();
  if !members.contains(&expl_manager) {
    members.push(expl_manager);
  }
  for member_id in members {
    let payload = finalbiome::tx()
      .organization_identity()
      .add_member(member_id);

    let _org_member = api
      .tx()
      .sign_and_submit_then_watch_default(&payload, &organization_signer)
      .await?
      .wait_for_in_block()
      .await?
      .wait_for_success()
      .await?;
  }

  // 3. Create FA
  // for the each fa id in the game spec we store an id of the created asset.

  // map stores the original and new id of the FA
  let mut fa_ids_map = HashMap::new();

  for (fa_id, fa_details) in game_spec.fa {
    let organization_id = MultiAddress::Id(organization_signer.account_id().clone());
    let payload = finalbiome::tx().fungible_assets().create(
      organization_id,
      fa_details.name.0,
      fa_details.top_upped,
      fa_details.cup_global,
      fa_details.cup_local,
    );

    let tx_fa_create = api
      .tx()
      .sign_and_submit_then_watch_default(&payload, &manager_signer)
      .await?;

    let fa_create = tx_fa_create
      .wait_for_in_block()
      .await?
      .wait_for_success()
      .await?;

    // lookup events and find asset id of the created asset
    let created_event = fa_create
      .find_first::<finalbiome::fungible_assets::events::Created>()?
      .ok_or_else(|| format!("Creating of FA {:?} failed", fa_id))?;

    fa_ids_map.insert(fa_id, created_event.asset_id);
  }

  println!("{:?}", fa_ids_map);

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
