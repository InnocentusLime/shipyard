use crate::all_storages::{AllStoragesBuilder, LockPresent, ThreadIdPresent};
use crate::atomic_refcell::AtomicRefCell;
use crate::public_transport::ShipyardRwLock;
use crate::world::World;
use alloc::sync::Arc;
use core::sync::atomic::AtomicU64;

/// Builder for [`World`] when one wants custom lock, custom thread pool
/// or custom thread id provider function.
pub struct WorldBuilder<Lock, ThreadId> {
    all_storages_builder: AllStoragesBuilder<Lock, ThreadId>,
}

impl World {
    /// Returns a builder for [`World`] when one wants custom lock, custom thread pool
    /// or custom thread id provider function.
    #[cfg(feature = "std")]
    pub fn builder() -> WorldBuilder<LockPresent, ThreadIdPresent> {
        WorldBuilder {
            all_storages_builder: AllStoragesBuilder::<LockPresent, ThreadIdPresent>::new(),
        }
    }

    /// Returns a builder for [`World`] when one wants custom lock, custom thread pool
    /// or custom thread id provider function.
    #[cfg(all(not(feature = "std"), not(feature = "thread_local")))]
    pub fn builder() -> WorldBuilder<crate::all_storages::MissingLock, ThreadIdPresent> {
        WorldBuilder {
            all_storages_builder: AllStoragesBuilder::<
                crate::all_storages::MissingLock,
                ThreadIdPresent,
            >::new(),
        }
    }

    /// Returns a builder for [`World`] when one wants custom lock, custom thread pool
    /// or custom thread id provider function.
    #[cfg(all(not(feature = "std"), feature = "thread_local"))]
    pub fn builder(
    ) -> WorldBuilder<crate::all_storages::MissingLock, crate::all_storages::MissingThreadId> {
        WorldBuilder {
            all_storages_builder: AllStoragesBuilder::<
                crate::all_storages::MissingLock,
                crate::all_storages::MissingThreadId,
            >::new(),
        }
    }
}

impl<Lock, ThreadId> WorldBuilder<Lock, ThreadId> {
    /// Use a custom `RwLock` for [`AllStorages`].
    ///
    /// [`AllStorages`]: crate::AllStorages
    pub fn with_custom_lock<L: ShipyardRwLock + Send + Sync>(
        self,
    ) -> WorldBuilder<LockPresent, ThreadId> {
        WorldBuilder {
            all_storages_builder: self.all_storages_builder.with_custom_lock::<L>(),
        }
    }
}

impl WorldBuilder<LockPresent, ThreadIdPresent> {
    /// Creates a new [`World`] based on the [`WorldBuilder`] config.
    pub fn build(self) -> World {
        let counter = Arc::new(AtomicU64::new(1));

        let all_storages = self.all_storages_builder.build(counter.clone());

        World {
            all_storages,
            scheduler: AtomicRefCell::new(Default::default()),
            counter,
        }
    }
}
