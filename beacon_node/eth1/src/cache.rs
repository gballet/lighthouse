use crate::error::{Error, Result};
use crate::types::Eth1DataFetcher;
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio;
use types::*;
use web3::futures::*;
use web3::types::*;

/// Cache for recent Eth1Data fetched from the Eth1 chain.
#[derive(Clone, Debug)]
pub struct BlockCache<F: Eth1DataFetcher> {
    cache: Arc<RwLock<BTreeMap<U256, Eth1Data>>>,
    last_block: Arc<RwLock<u64>>,
    fetcher: Arc<F>,
}

impl<F: Eth1DataFetcher> BlockCache<F> {
    pub fn new(fetcher: Arc<F>) -> Self {
        BlockCache {
            cache: Arc::new(RwLock::new(BTreeMap::new())),
            // Note: Should ideally start from block where Eth1 chain starts accepting deposits.
            last_block: Arc::new(RwLock::new(0)),
            fetcher: fetcher,
        }
    }

    /// Called periodically to populate the cache with Eth1Data
    /// from most recent blocks upto `distance`.
    pub fn update_cache(&self, distance: u64) -> impl Future<Item = (), Error = Error> + Send {
        let cache_updated = self.cache.clone();
        let last_block = self.last_block.clone();
        let fetcher = self.fetcher.clone();
        let future = self
            .fetcher
            .get_current_block_number()
            .and_then(move |curr_block_number| {
                fetch_eth1_data_in_range(0, distance, curr_block_number, fetcher)
                    .for_each(move |data| {
                        let data = data?;
                        let mut eth1_cache = cache_updated.write();
                        eth1_cache.insert(data.0, data.1);
                        Ok(())
                    })
                    .and_then(move |_| {
                        let mut last_block_updated = last_block.write();
                        *last_block_updated = curr_block_number.as_u64();
                        // TODO: Delete older stuff
                        Ok(())
                    })
            });
        future
    }

    /// Get `Eth1Data` object at a distance of `distance` from the perceived head of the currrent Eth1 chain.
    /// Returns the object from the cache if present, else fetches from Eth1Fetcher.
    pub fn get_eth1_data(&self, distance: u64) -> Result<Eth1Data> {
        let current_block_number: U256 =
            tokio::runtime::current_thread::block_on_all(self.fetcher.get_current_block_number())?;
        let block_number: U256 = current_block_number
            .checked_sub(distance.into())
            .unwrap_or(U256::zero());
        if let Some(result) = self.cache.read().get(&block_number) {
            return Ok(result.clone());
        } else {
            // Note: current_thread::block_on_all() might not be safe here since
            // it waits for other spawned futures to complete on current thread.
            if let Ok((block_number, eth1_data)) = tokio::runtime::current_thread::block_on_all(
                fetch_eth1_data(distance, current_block_number, self.fetcher.clone()),
            )? {
                let mut cache_write = self.cache.write();
                cache_write.insert(block_number, eth1_data.clone());
                return Ok(eth1_data);
            } else {
                // Note: Should never reach here
                return Err(Error::Web3Error(web3::error::Error::InvalidResponse(
                    "Failed to fetch eth1 data".to_string(),
                )));
            }
        }
    }

    /// Returns a Vec<Eth1Data> corresponding to given distance range.
    pub fn get_eth1_data_in_range(&self, start: u64, end: u64) -> Vec<Eth1Data> {
        (start..end)
            .map(|h| self.get_eth1_data(h))
            .flatten() // Chuck Err values. This might be okay since its unlikely that the entire range returns None.
            .collect::<Vec<Eth1Data>>()
    }
}

fn fetch_eth1_data_in_range<F: Eth1DataFetcher>(
    start: u64,
    end: u64,
    current_block_number: U256,
    fetcher: Arc<F>,
) -> impl Stream<Item = Result<(U256, Eth1Data)>, Error = Error> + Send {
    stream::futures_ordered(
        (start..end).map(move |i| fetch_eth1_data(i, current_block_number, fetcher.clone())),
    )
}

/// Fetches Eth1 data from the Eth1Data fetcher object.
fn fetch_eth1_data<F: Eth1DataFetcher>(
    distance: u64,
    current_block_number: U256,
    fetcher: Arc<F>,
) -> impl Future<Item = Result<(U256, Eth1Data)>, Error = Error> + Send {
    let block_number: U256 = current_block_number
        .checked_sub(distance.into())
        .unwrap_or(U256::zero());
    let deposit_root = fetcher.get_deposit_root(Some(BlockNumber::Number(block_number.as_u64())));
    let deposit_count = fetcher.get_deposit_count(Some(BlockNumber::Number(block_number.as_u64())));
    let block_hash = fetcher.get_block_hash_by_height(block_number.as_u64());
    let eth1_data_future = deposit_root.join3(deposit_count, block_hash);
    eth1_data_future.map(move |data| {
        let eth1_data = Eth1Data {
            deposit_root: data.0,
            deposit_count: data.1?,
            block_hash: data
                .2
                .ok_or(Error::Web3Error(web3::error::Error::InvalidResponse(
                    "Block at given height does not exist".to_string(),
                )))?,
        };
        Ok((block_number, eth1_data))
    })
}

#[cfg(all(test, feature = "integration_tests"))]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::web3_fetcher::Web3DataFetcher;
    use slog::{o, Drain};
    use std::time::{Duration, Instant};
    use tokio::timer::{Delay, Interval};

    fn setup_log() -> slog::Logger {
        let decorator = slog_term::TermDecorator::new().build();
        let drain = slog_term::FullFormat::new(decorator).build().fuse();
        let drain = slog_async::Async::new(drain).build().fuse();
        slog::Logger::root(drain, o!())
    }

    fn setup() -> Web3DataFetcher {
        let config = Config::default();
        let w3 = Web3DataFetcher::new(
            &config.endpoint,
            &config.address,
            config.timeout,
            &setup_log(),
        );
        return w3.unwrap();
    }

    #[test]
    fn test_fetch() {
        let w3 = setup();
        let when = Instant::now() + Duration::from_millis(5000);
        let task1 = Delay::new(when)
            .and_then(|_| Ok(()))
            .map_err(|e| panic!("delay errored; err={:?}", e));
        tokio::run(task1);
        let task2 = fetch_eth1_data(0, 10.into(), Arc::new(w3)).and_then(|data| {
            assert!(data.is_ok(), "Failed to fetch eth1 data");
            Ok(())
        });
        tokio::run(task2.map_err(|e| panic!("{:?}", e)));
    }

    #[test]
    fn test_cache() {
        let w3 = setup();
        let interval = {
            let update_duration = Duration::from_secs(15);
            Interval::new(Instant::now(), update_duration).map_err(|e| panic!("{:?}", e))
        };

        let cache = BlockCache::new(Arc::new(w3));
        let cache_inside = cache.cache.clone();
        let task = interval.take(100).for_each(move |_| {
            let _c = cache_inside.clone();
            cache
                .update_cache(4)
                .and_then(move |_| Ok(()))
                .map_err(|e| panic!("Failed to update cache {:?}", e))
        });
        tokio::run(task.map_err(|e| panic!("{:?}", e)));
    }
}