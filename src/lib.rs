use game_spec::GameSpec;
use parity_scale_codec::Decode;
use sp_core::{
  crypto::{ExposeSecret, SecretString, Ss58Codec},
  hexdisplay::HexDisplay,
};
use sp_runtime::{
  self,
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
use subxt::tx::Signer;

use crate::{
  finalbiome::runtime_types::pallet_support::{characteristics::Characteristic, Attribute},
  game_spec::GameSpecBuilder,
  utils::{submit_default, AllKeyIter},
};

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
    derive = "serde::Serialize, serde::Deserialize, Debug, Hash, Copy"
  ),
  derive_for_type(
    type = "pallet_support::types::fungible_asset_balance::FungibleAssetBalance",
    derive = "serde::Serialize, serde::Deserialize"
  ),
  derive_for_type(
    type = "pallet_support::types::non_fungible_class_id::NonFungibleClassId",
    derive = "serde::Serialize, serde::Deserialize, Debug, Hash, Copy"
  ),
  derive_for_type(
    type = "pallet_support::types_nfa::ClassDetails",
    derive = "serde::Serialize, serde::Deserialize"
  ),
  derive_for_type(
    type = "pallet_support::characteristics::bettor::Bettor",
    derive = "serde::Serialize, serde::Deserialize"
  ),
  derive_for_type(
    type = "pallet_support::characteristics::bettor::BettorOutcome",
    derive = "serde::Serialize, serde::Deserialize"
  ),
  derive_for_type(
    type = "pallet_support::characteristics::bettor::BettorWinning",
    derive = "serde::Serialize, serde::Deserialize"
  ),
  derive_for_type(
    type = "pallet_support::characteristics::bettor::DrawOutcomeResult",
    derive = "serde::Serialize, serde::Deserialize"
  ),
  derive_for_type(
    type = "pallet_support::characteristics::bettor::OutcomeResult",
    derive = "serde::Serialize, serde::Deserialize"
  ),
  derive_for_type(
    type = "pallet_support::characteristics::purchased::Purchased",
    derive = "serde::Serialize, serde::Deserialize"
  ),
  derive_for_type(
    type = "pallet_support::characteristics::purchased::Offer",
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

pub(crate) type NonFungibleClassId =
  finalbiome::runtime_types::pallet_support::types::non_fungible_class_id::NonFungibleClassId;

pub(crate) type NonFungibleDetails =
  finalbiome::runtime_types::pallet_support::types_nfa::ClassDetails<AccountId32>;

pub(crate) type FungibleAssetIds = Vec<(FungibleAssetId, FungibleAssetDetails)>;
pub(crate) type NonFungibleClassDetails = Vec<(NonFungibleClassId, NonFungibleDetails)>;
pub(crate) type AttributeKey =
  finalbiome::runtime_types::sp_runtime::bounded::bounded_vec::BoundedVec<u8>;
pub(crate) type AttributesDetails = Vec<(
  NonFungibleClassId,
  AttributeKey,
  finalbiome::runtime_types::pallet_support::AttributeValue,
)>;

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
  let nfas = fetch_nfas(&api, &organization_id, block_hash);
  let attrs = fetch_nfa_attributes(&api, &organization_id, block_hash);

  let game_spec_builder = GameSpecBuilder::new();
  let game_spec = game_spec_builder
    .version(node_version.await?)
    .hash(format!("0x{}", HexDisplay::from(&block_hash.as_ref())))
    .organization_details(org_details.await?)
    .organization_members(org_members.await?)
    .fa(fas.await?)
    .nfa(nfas.await?)
    .attributes(attrs.await?)
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
  // Iterate over the membersOf storage to get all fa of the game
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
    let address = finalbiome::storage().fungible_assets().assets(asset_id);
    let details: FungibleAssetDetails = api
      .storage()
      .fetch(&address, Some(block_hash))
      .await?
      .ok_or_else(|| format!("FA {:?} not found", asset_id))?;

    fa_details.push((asset_id, details));
  }

  Ok(fa_details)
}

/// Fetch the nfa ids belonging to the game
async fn fetch_nfa_ids<T>(
  api: &OnlineClient<T>,
  organization_id: &AccountId32,
  block_hash: T::Hash,
) -> ResultOf<Vec<NonFungibleClassId>>
where
  T: subxt::Config,
{
  let mut class_ids = vec![];
  // Iterate over the classAccounts storage to get all nfa of the game
  let key_addr = finalbiome::storage()
    .non_fungible_assets()
    .class_accounts_root();
  // Obtain the root bytes
  let mut query_key = key_addr.to_root_bytes();
  // We know that the first key is a AccountId32 and is hashed by Blake2_128Concat.
  // We can build a `StorageMapKey` that replicates that, and append those bytes to the above.
  StorageMapKey::new(organization_id, StorageHasher::Blake2_128Concat).to_bytes(&mut query_key);
  // Iterate over keys at that address and collect values to vec
  let mut iter = AllKeyIter::new(api, query_key, block_hash, 10);
  while let Some(key) = iter.next().await? {
    // we need last 4 bytes (u32) - the asset id.
    let class_id_encoded = key.0.as_slice()[key.0.len() - 4..].to_vec();
    let class_id =
      finalbiome::runtime_types::pallet_support::types::non_fungible_class_id::NonFungibleClassId::decode(
        &mut &*class_id_encoded,
      )?;

    // println!("Member of {} is {}", &organization_id.to_ss58check(), &member_id.to_ss58check());
    class_ids.push(class_id);
  }
  Ok(class_ids)
}

async fn fetch_nfas<T>(
  api: &OnlineClient<T>,
  organization_id: &AccountId32,
  block_hash: T::Hash,
) -> ResultOf<NonFungibleClassDetails>
where
  T: subxt::Config,
{
  // 1. Fetch the nfa ids belonging to the game
  let class_ids = fetch_nfa_ids(api, organization_id, block_hash).await?;

  // 2. Fetch details about each found assets
  let mut nfa_details = vec![];
  for class_id in class_ids {
    let address = finalbiome::storage()
      .non_fungible_assets()
      .classes(class_id);
    let details = api
      .storage()
      .fetch(&address, Some(block_hash))
      .await?
      .ok_or_else(|| format!("NFA {:?} not found", class_id))?;

    nfa_details.push((class_id, details));
  }

  Ok(nfa_details)
}

/// Fetch all attributes keys for given nfa
async fn fetch_nfa_attributes_ids<T>(
  api: &OnlineClient<T>,
  organization_id: &AccountId32,
  block_hash: T::Hash,
  nfa_id: NonFungibleClassId,
) -> ResultOf<Vec<AttributeKey>>
where
  T: subxt::Config,
{
  let mut attrs_keys = vec![];
  // Iterate over the classAttributes storage to get all nfa attrs of the nfa
  let key_addr = finalbiome::storage()
    .non_fungible_assets()
    .class_attributes_root();
  // Obtain the root bytes
  let mut query_key = key_addr.to_root_bytes();

  // We know that the first key is a NonFungibleClassId and is hashed by Blake2_128Concat.
  // We can build a `StorageMapKey` that replicates that, and append those bytes to the above.
  StorageMapKey::new(nfa_id, StorageHasher::Blake2_128Concat).to_bytes(&mut query_key);
  let partial_length = query_key.len();
  // Iterate over keys at that address and collect values to vec
  let mut iter = AllKeyIter::new(api, query_key, block_hash, 10);
  while let Some(key) = iter.next().await? {
    // we need all bytes after `partial_length` + 16(Blake2_128Concat) - the attr id (key).
    let attr_key_encoded = key.0.as_slice()[partial_length + 16..].to_vec();
    let attr_key = AttributeKey::decode(&mut &*attr_key_encoded)?;

    println!(
      "Attrs of {} is {}",
      &organization_id.to_ss58check(),
      std::str::from_utf8(&attr_key.0).expect("attr key is text")
    );
    attrs_keys.push(attr_key);
  }
  Ok(attrs_keys)
}

async fn fetch_nfa_attributes<T>(
  api: &OnlineClient<T>,
  organization_id: &AccountId32,
  block_hash: T::Hash,
) -> ResultOf<AttributesDetails>
where
  T: subxt::Config,
{
  // 1. Fetch the nfa ids belonging to the game
  let class_ids = fetch_nfa_ids(api, organization_id, block_hash).await?;

  // 2. Fetch all attributes for all nfas
  let mut attributes = vec![];
  for class_id in class_ids {
    let attr_keys = fetch_nfa_attributes_ids(api, organization_id, block_hash, class_id).await?;
    for attr_key in attr_keys {
      let address = finalbiome::storage()
        .non_fungible_assets()
        .class_attributes(class_id, &attr_key);
      let attr_value = api
        .storage()
        .fetch(&address, Some(block_hash))
        .await?
        .ok_or_else(|| format!("NFA Attr {:?} for NFA {:?} not found", attr_key, class_id))?;

      attributes.push((class_id, attr_key, attr_value));
    }
  }
  Ok(attributes)
}

/// Creates an appropriate game configuration in the network
async fn post_to_node<T, P>(
  api: &OnlineClient<T>,
  game_spec: GameSpec,
  organization_signer: PairSigner<T, P>,
  manager_signer: PairSigner<T, P>,
) -> ResultOf<()>
where
T: subxt::Config<AccountId = AccountId32>,
P: sp_core::Pair,
<<T as subxt::Config>::ExtrinsicParams as subxt::tx::ExtrinsicParams<
  <T as subxt::Config>::Index,
  <T as subxt::Config>::Hash,
>>::OtherParams: std::default::Default,
<T as subxt::Config>::Address: std::convert::From<<T as subxt::Config>::AccountId>,
<T as subxt::Config>::Signature: std::convert::From<<P as sp_core::Pair>::Signature>,
{
  // todo: make transactional creation of the configuration in the network

  let org_name = game_spec.clone().organization_details.name.0;

  // 1. Create organization
  let payload = finalbiome::tx()
    .organization_identity()
    .create_organization(org_name);

  submit_default(api, &payload, &organization_signer).await?;

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

    submit_default(api, &payload, &organization_signer).await?;
  }

  // 3. Create FA
  // for the each fa id in the game spec we store an id of the created asset.

  // map stores the original and new id of the FA
  let mut fa_ids_map = HashMap::new();

  for (fa_id, fa_details) in game_spec.clone().fa {
    let organization_id = MultiAddress::Id(organization_signer.account_id().clone());
    let payload = finalbiome::tx().fungible_assets().create(
      organization_id,
      fa_details.name.0,
      fa_details.top_upped,
      fa_details.cup_global,
      fa_details.cup_local,
    );

    let fa_create = submit_default(api, &payload, &manager_signer).await?;

    // lookup events and find asset id of the created asset
    let created_event = fa_create
      .find_first::<finalbiome::fungible_assets::events::Created>()?
      .ok_or_else(|| format!("Creating of FA {:?} failed", fa_id))?;

    fa_ids_map.insert(fa_id, created_event.asset_id);
  }

  println!("{:?}", fa_ids_map);

  // 4. Create NFA
  // for the each fa id in the game spec we store an id of the created asset.

  // map stores the original and new id of the FA
  let mut nfa_ids_map = HashMap::new();

  for (nfa_id_orig, nfa_details) in game_spec.clone().nfa {
    let organization_id = MultiAddress::Id(organization_signer.account_id().clone());
    // 1. Creata nfa
    let payload = finalbiome::tx()
      .non_fungible_assets()
      .create(organization_id.clone(), nfa_details.name.0);
    let nfa_create = submit_default(api, &payload, &manager_signer).await?;
    // lookup events and find asset id of the created asset
    let created_event = nfa_create
      .find_first::<finalbiome::non_fungible_assets::events::Created>()?
      .ok_or_else(|| format!("Creating of NFA {:?} failed", nfa_id_orig))?;
    let nfa_id_created = created_event.class_id;
    nfa_ids_map.insert(nfa_id_orig, nfa_id_created);

    println!("NFA {:?} Created", nfa_id_orig);

    // 2. Add attributes
    let attrs = game_spec
      .attributes
      .clone()
      .into_iter()
      .filter(|(class, ..)| class == &nfa_id_orig);
    for (_class_id, key, value) in attrs {
      let attr = Attribute { key, value };
      let payload = finalbiome::tx().non_fungible_assets().create_attribute(
        organization_id.clone(),
        nfa_id_created,
        attr,
      );
      submit_default(api, &payload, &manager_signer).await?;
    }

    // 3. Set characteristics
    // 3.1 Bettor
    let nfa_spec = game_spec.clone().get_nfa(nfa_id_orig);
    if let Some(bettor) = nfa_spec.bettor {
      let characteristic = Characteristic::Bettor(Some(bettor));
      let payload = finalbiome::tx().non_fungible_assets().set_characteristic(
        organization_id.clone(),
        nfa_id_created,
        characteristic,
      );
      submit_default(api, &payload, &manager_signer).await?;
    }
    // 3.2 Purchased
    if let Some(purchased) = nfa_spec.purchased {
      let characteristic = Characteristic::Purchased(Some(purchased));
      let payload = finalbiome::tx().non_fungible_assets().set_characteristic(
        organization_id.clone(),
        nfa_id_created,
        characteristic,
      );
      submit_default(api, &payload, &manager_signer).await?;
    }
  }

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
