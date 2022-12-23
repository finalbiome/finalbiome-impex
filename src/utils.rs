use sp_core::storage::StorageKey;
use subxt::{
  tx::{PairSigner, StaticTxPayload},
  OnlineClient, blocks::ExtrinsicEvents,
};

use crate::ResultOf;

/// Iterates over all keys in a map by prefix key
pub(crate) struct AllKeyIter<'a, T>
where
  T: subxt::Config,
{
  api: &'a OnlineClient<T>,
  query_key: Vec<u8>,
  page_size: u32,
  block_hash: T::Hash,
  start_key: Option<StorageKey>,
  buffer: Vec<StorageKey>,
}

impl<'a, T> AllKeyIter<'a, T>
where
  T: subxt::Config,
{
  /// Create new iterator
  pub fn new(
    api: &OnlineClient<T>,
    query_key: Vec<u8>,
    block_hash: T::Hash,
    page_size: u32,
  ) -> AllKeyIter<T> {
    AllKeyIter {
      api,
      query_key,
      page_size,
      block_hash,
      start_key: None,
      buffer: Default::default(),
    }
  }
  /// Returns the next key from a storage.
  pub async fn next(&mut self) -> ResultOf<Option<StorageKey>> {
    loop {
      if let Some(k) = self.buffer.pop() {
        return Ok(Some(k));
      } else {
        let start_key = self.start_key.take();
        let mut keys = self
          .api
          .storage()
          .fetch_keys(
            &self.query_key,
            self.page_size,
            start_key.as_ref().map(|k| &*k.0),
            Some(self.block_hash),
          )
          .await?;

        if keys.is_empty() {
          return Ok(None);
        }

        self.start_key = keys.last().cloned();
        self.buffer.append(&mut keys);
      }
    }
  }
}

/// Submit Tx payload with default settings.
pub(crate) async fn submit_default<T, C, P>(
  api: &OnlineClient<T>,
  payload: &StaticTxPayload<C>,
  signer: &PairSigner<T, P>,
) -> ResultOf<ExtrinsicEvents<T>>
where
  T: subxt::Config,
  P: sp_core::Pair,
  C: parity_scale_codec::Encode,
  <<T as subxt::Config>::ExtrinsicParams as subxt::tx::ExtrinsicParams<
    <T as subxt::Config>::Index,
    <T as subxt::Config>::Hash,
  >>::OtherParams: std::default::Default,
  <T as subxt::Config>::Address: std::convert::From<<T as subxt::Config>::AccountId>,
  <T as subxt::Config>::Signature: std::convert::From<<P as sp_core::Pair>::Signature>,
{
  let events = api
    .tx()
    .sign_and_submit_then_watch_default(payload, signer)
    .await?
    .wait_for_in_block()
    .await?
    .wait_for_success()
    .await?;
  Ok(events)
}
