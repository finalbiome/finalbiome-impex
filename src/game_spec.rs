use serde::{Deserialize, Serialize};

use crate::{
  AttributesDetails, FungibleAssetIds, NonFungibleClassDetails, NonFungibleClassId,
  NonFungibleDetails, OrganizationDetails,
};

type OrganizationMembers = Vec<sp_runtime::AccountId32>;

#[derive(Serialize, Deserialize, Clone)]
/// A FinalBiome game spec struct which holds configuration of the game.
pub struct GameSpec {
  /// Version of the node
  pub version: String,
  /// Hash of the network block when the game spec was exported
  pub hash: String,
  /// Game details
  pub organization_details: OrganizationDetails,
  /// Members of the organization
  pub organization_members: OrganizationMembers,
  /// Fungible assets
  pub fa: FungibleAssetIds,
  /// Non fungible assets
  pub nfa: NonFungibleClassDetails,
  /// NFA Attributes
  pub attributes: AttributesDetails,
}

impl GameSpec {
  /// Returns nfa by given id
  pub fn get_nfa(&self, id: NonFungibleClassId) -> NonFungibleDetails {
    self
      .nfa
      .iter()
      .find(|(class_id, ..)| class_id == &id)
      .map(|(_class_id, details)| details.clone())
      .unwrap_or_else(|| panic!("cannot find class with given id: {:?}", id))
  }
}

#[derive(Default)]
pub(crate) struct GameSpecBuilder {
  /// Version of the node
  pub version: String,
  /// Hash of the network block when the game spec was exported
  pub hash: String,
  /// Game details
  pub organization_details: Option<OrganizationDetails>,
  /// Members of the organization
  pub organization_members: Option<OrganizationMembers>,
  /// Fungible assets
  pub fa: Option<FungibleAssetIds>,
  /// Non fungible assets
  pub nfa: Option<NonFungibleClassDetails>,
  /// NFA Attributes
  pub attributes: Option<AttributesDetails>,
}

impl GameSpecBuilder {
  pub fn new() -> GameSpecBuilder {
    GameSpecBuilder::default()
  }

  /// Set version of the node
  pub fn version(mut self, version: String) -> GameSpecBuilder {
    self.version = version;
    self
  }
  /// Set hash of the state
  pub fn hash(mut self, hash: String) -> GameSpecBuilder {
    self.hash = hash;
    self
  }

  /// Set organization details
  pub fn organization_details(
    mut self,
    organization_details: OrganizationDetails,
  ) -> GameSpecBuilder {
    self.organization_details = Some(organization_details);
    self
  }

  /// Set members of the organization
  pub fn organization_members(
    mut self,
    organization_members: OrganizationMembers,
  ) -> GameSpecBuilder {
    self.organization_members = Some(organization_members);
    self
  }

  /// Set FA of the organization
  pub fn fa(mut self, fa: FungibleAssetIds) -> GameSpecBuilder {
    self.fa = Some(fa);
    self
  }

  /// Set NFA of the organization
  pub fn nfa(mut self, nfa: NonFungibleClassDetails) -> GameSpecBuilder {
    self.nfa = Some(nfa);
    self
  }

  /// Set NFA Attributes of the organization
  pub fn attributes(mut self, attributes: AttributesDetails) -> GameSpecBuilder {
    self.attributes = Some(attributes);
    self
  }

  pub fn try_build(self) -> Result<GameSpec, Box<dyn std::error::Error>> {
    if self.organization_details.is_none() {
      return Err("Organization details not set".into());
    };

    Ok(GameSpec {
      version: self.version,
      hash: self.hash,
      organization_details: self.organization_details.expect("org details exists"),
      organization_members: self.organization_members.expect("org members exists"),
      fa: self.fa.expect("fa exists"),
      nfa: self.nfa.expect("nfa exists"),
      attributes: self.attributes.expect("nfa attrs exists"),
    })
  }
}
