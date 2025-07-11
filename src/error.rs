//! All error types.

use crate::entity_id::EntityId;
use crate::storage::StorageId;
use alloc::borrow::Cow;
use alloc::boxed::Box;
use core::fmt::{Debug, Display, Formatter};
#[cfg(feature = "std")]
use std::error::Error;

/// AtomicRefCell's borrow error.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Borrow {
    /// The Storage was borrowed when an exclusive borrow occurred.
    Unique,
    /// The Storage was borrowed exclusively when a shared borrow occurred.
    Shared,
    /// The Storage of a `!Send` component was accessed from an other thread.
    WrongThread,
    /// The Storage of a `!Sync` component was accessed from multiple threads at the same time.
    MultipleThreads,
}

#[cfg(feature = "std")]
impl Error for Borrow {}

impl Debug for Borrow {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        match self {
            Borrow::Unique => f.write_str("Cannot mutably borrow while already borrowed."),
            Borrow::Shared => {
                f.write_str("Cannot immutably borrow while already mutably borrowed.")
            }
            Borrow::WrongThread => {
                f.write_str("Can't access from another thread because it's !Send and !Sync.")
            }
            Borrow::MultipleThreads => f.write_str(
                "Can't access from multiple threads at the same time because it's !Sync.",
            ),
        }
    }
}

impl Display for Borrow {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        Debug::fmt(self, f)
    }
}

/// Error related to acquiring a storage.
pub enum GetStorage {
    #[allow(missing_docs)]
    AllStoragesBorrow(Borrow),
    #[allow(missing_docs)]
    StorageBorrow {
        name: Option<&'static str>,
        id: StorageId,
        borrow: Borrow,
    },
    #[allow(missing_docs)]
    Entities(Borrow),
    #[allow(missing_docs)]
    MissingStorage {
        name: Option<&'static str>,
        id: StorageId,
    },
    #[allow(missing_docs)]
    TrackingNotEnabled {
        name: Option<&'static str>,
        id: StorageId,
        tracking: &'static str,
    },
    /// Error returned by a custom view.
    #[cfg(feature = "std")]
    Custom(Box<dyn Error + Send + Sync>),
    /// Error returned by a custom view.
    #[cfg(not(feature = "std"))]
    Custom(Box<dyn core::any::Any + Send>),
}

impl GetStorage {
    #[cfg(feature = "std")]
    #[allow(missing_docs)]
    pub fn from_custom<E: Into<Box<dyn Error + Send + Sync>>>(error: E) -> GetStorage {
        GetStorage::Custom(error.into())
    }
    #[cfg(not(feature = "std"))]
    #[allow(missing_docs)]
    pub fn from_custom<E: core::any::Any + Send>(error: E) -> GetStorage {
        GetStorage::Custom(Box::new(error))
    }
}

impl PartialEq for GetStorage {
    fn eq(&self, other: &GetStorage) -> bool {
        match (self, other) {
            (GetStorage::AllStoragesBorrow(l0), GetStorage::AllStoragesBorrow(r0)) => l0 == r0,
            (
                GetStorage::StorageBorrow {
                    name: l_name,
                    id: l_id,
                    borrow: l_borrow,
                },
                GetStorage::StorageBorrow {
                    name: r_name,
                    id: r_id,
                    borrow: r_borrow,
                },
            ) => l_name == r_name && l_id == r_id && l_borrow == r_borrow,
            (GetStorage::Entities(l0), GetStorage::Entities(r0)) => l0 == r0,
            (
                GetStorage::MissingStorage {
                    name: l_name,
                    id: l_id,
                },
                GetStorage::MissingStorage {
                    name: r_name,
                    id: r_id,
                },
            ) => l_name == r_name && l_id == r_id,
            (
                GetStorage::TrackingNotEnabled {
                    name: l_name,
                    id: l_id,
                    tracking: l_tracking,
                },
                GetStorage::TrackingNotEnabled {
                    name: r_name,
                    id: r_id,
                    tracking: r_tracking,
                },
            ) => l_name == r_name && l_id == r_id && l_tracking == r_tracking,
            _ => false,
        }
    }
}

impl Eq for GetStorage {}

#[cfg(feature = "std")]
impl Error for GetStorage {}

impl Debug for GetStorage {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        match self {
            GetStorage::AllStoragesBorrow(borrow) => match borrow {
                Borrow::Unique => f.write_str("Cannot mutably borrow AllStorages while it's already borrowed (AllStorages is borrowed to access any storage)."),
                Borrow::Shared => {
                    f.write_str("Cannot immutably borrow AllStorages while it's already mutably borrowed.")
                },
                _ => unreachable!(),
            },
            GetStorage::StorageBorrow {name, id, borrow} => if let Some(name) = name {
                match borrow {
                    Borrow::Unique => f.write_fmt(format_args!("Cannot mutably borrow {} storage while it's already borrowed.", name)),
                    Borrow::Shared => {
                        f.write_fmt(format_args!("Cannot immutably borrow {} storage while it's already mutably borrowed.", name))
                    },
                    Borrow::MultipleThreads => f.write_fmt(format_args!("Cannot borrow {} storage from multiple thread at the same time because it's !Sync.", name)),
                    Borrow::WrongThread => f.write_fmt(format_args!("Cannot borrow {} storage from other thread than the one it was created in because it's !Send and !Sync.", name)),
                }
            } else {
                match borrow {
                    Borrow::Unique => f.write_fmt(format_args!("Cannot mutably borrow {:?} storage while it's already borrowed.", id)),
                    Borrow::Shared => {
                        f.write_fmt(format_args!("Cannot immutably borrow {:?} storage while it's already mutably borrowed.", id))
                    },
                    Borrow::MultipleThreads => f.write_fmt(format_args!("Cannot borrow {:?} storage from multiple thread at the same time because it's !Sync.", id)),
                    Borrow::WrongThread => f.write_fmt(format_args!("Cannot borrow {:?} storage from other thread than the one it was created in because it's !Send and !Sync.", id)),
                }
            }
            GetStorage::Entities(borrow) => match borrow {
                Borrow::Unique => f.write_str("Cannot mutably borrow Entities storage while it's already borrowed."),
                Borrow::Shared => {
                    f.write_str("Cannot immutably borrow Entities storage while it's already mutably borrowed.")
                },
                _ => unreachable!(),
            },
            GetStorage::MissingStorage { name, id } => if let Some(name) = name {
                f.write_fmt(format_args!("{} storage was not found in the World. You can register unique storage with: world.add_unique(your_unique);", name))
            } else {
                f.write_fmt(format_args!("{:?} storage was not found in the World. You can register unique storage with: world.add_unique(your_unique);", id))
            }
            GetStorage::TrackingNotEnabled { name, id, tracking } => if let Some(name) = name {
                f.write_fmt(format_args!("{} tracking is not enabled for {} storage.", tracking, name))
            } else {
                f.write_fmt(format_args!("{} tracking is not enabled for {:?} storage.", tracking, id))
            }
            GetStorage::Custom(err) => {
                f.write_fmt(format_args!("Storage borrow failed with a custom error, {:?}.", err))
            }
        }
    }
}

impl Display for GetStorage {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        Debug::fmt(self, f)
    }
}

/// Error related to adding an entity.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NewEntity {
    /// Another add_storage operation is in progress.
    AllStoragesBorrow(Borrow),
    /// Entities is already borrowed.
    Entities(Borrow),
}

#[cfg(feature = "std")]
impl Error for NewEntity {}

impl Debug for NewEntity {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        match self {
            NewEntity::AllStoragesBorrow(borrow) => match borrow {
                Borrow::Unique => f.write_str("Cannot mutably borrow all storages while it's already borrowed (this include component storage)."),
                Borrow::Shared => {
                    f.write_str("Cannot immutably borrow all storages while it's already mutably borrowed.")
                },
                _ => unreachable!(),
            },
            NewEntity::Entities(borrow) => match borrow {
                Borrow::Unique => f.write_str("Cannot mutably borrow entities while it's already borrowed."),
                _ => unreachable!(),
            },
        }
    }
}

impl Display for NewEntity {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        Debug::fmt(self, f)
    }
}

/// Returned by [`AllStorages::add_component`] and [`World::add_component`] when trying to add components to an entity that is not alive.
///
/// [`AllStorages::add_component`]: crate::all_storages::AllStorages::add_component()
/// [`World::add_component`]: crate::world::World::add_component()
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AddComponent {
    #[allow(missing_docs)]
    EntityIsNotAlive,
}

#[cfg(feature = "std")]
impl Error for AddComponent {}

impl Debug for AddComponent {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        match self {
            AddComponent::EntityIsNotAlive => {
                f.write_str("Entity has to be alive to add component to it.")
            }
        }
    }
}

impl Display for AddComponent {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        Debug::fmt(self, f)
    }
}

/// Error returned by [`World::run`] and [`AllStorages::run`].
/// Can refer to an invalid storage borrow or a custom error.
///
/// [`World::run`]: crate::World::run()
/// [`AllStorages::run`]: crate::AllStorages::run()
pub enum Run {
    /// Failed to borrow one of the storage.
    GetStorage(GetStorage),
    /// Error returned by the system.
    #[cfg(feature = "std")]
    Custom(Box<dyn Error + Send + Sync>),
    /// Error returned by the system.
    #[cfg(not(feature = "std"))]
    Custom(Box<dyn core::any::Any + Send>),
}

impl From<GetStorage> for Run {
    fn from(get_storage: GetStorage) -> Run {
        Run::GetStorage(get_storage)
    }
}

impl Run {
    #[cfg(feature = "std")]
    #[allow(missing_docs)]
    pub fn from_custom<E: Into<Box<dyn Error + Send + Sync>>>(error: E) -> Run {
        Run::Custom(error.into())
    }
    #[cfg(not(feature = "std"))]
    #[allow(missing_docs)]
    pub fn from_custom<E: core::any::Any + Send>(error: E) -> Run {
        Run::Custom(Box::new(error))
    }
}

impl PartialEq for Run {
    fn eq(&self, other: &Run) -> bool {
        match (self, other) {
            (Run::GetStorage(l_get_storage), Run::GetStorage(r_get_storage)) => {
                l_get_storage == r_get_storage
            }
            _ => false,
        }
    }
}

#[cfg(feature = "std")]
impl Error for Run {}

impl Debug for Run {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        match self {
            Run::GetStorage(get_storage) => Debug::fmt(&get_storage, f),
            Run::Custom(err) => {
                f.write_fmt(format_args!("run failed with a custom error, {:?}.", err))
            }
        }
    }
}

impl Display for Run {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        Debug::fmt(self, f)
    }
}

/// Returned by [`get`] when an entity does not have a component in the requested storage(s).
///
/// [`get`]: crate::Get
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct MissingComponent {
    /// `EntityId` of the component.
    pub id: EntityId,
    /// Name of the component.
    pub name: &'static str,
}

#[cfg(feature = "std")]
impl Error for MissingComponent {}

impl Debug for MissingComponent {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        f.write_fmt(format_args!(
            "{:?} does not have a {} component.",
            self.id, self.name
        ))
    }
}

impl Display for MissingComponent {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        Debug::fmt(self, f)
    }
}

/// Returned when trying to add an invalid system to a workload.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum InvalidSystem {
    /// `AllStorages` borrowed alongside another storage.
    AllStorages,
    /// Multiple views of the same storage including an exclusive one.
    MultipleViews,
    /// Multiple exclusive views for the same storage.
    MultipleViewsMut,
    /// System returning `Workload`
    WorkloadUsedAsSystem(&'static str),
}

#[cfg(feature = "std")]
impl Error for InvalidSystem {}

impl Debug for InvalidSystem {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        match self {
            InvalidSystem::AllStorages => f.write_str("A system borrowing both AllStorages and a storage can't run. You can borrow the storage inside the system with AllStorages::borrow or AllStorages::run instead."),
            InvalidSystem::MultipleViews => f.write_str("Multiple views of the same storage including an exclusive borrow, consider removing the shared borrow."),
            InvalidSystem::MultipleViewsMut => f.write_str("Multiple exclusive views of the same storage, consider removing one."),
            InvalidSystem::WorkloadUsedAsSystem(system_name) => f.write_fmt(format_args!("Workload used as a system, you should call it `{}()`.", system_name)),
        }
    }
}

impl Display for InvalidSystem {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        Debug::fmt(self, f)
    }
}

/// Error returned by [`World::remove_unique`] and [`AllStorages::remove_unique`].
///
/// [`World::remove_unique`]: crate::World::remove_unique()
/// [`AllStorages::remove_unique`]: crate::AllStorages::remove_unique()
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum UniqueRemove {
    /// `AllStorages` was already borrowed.
    AllStorages,
    /// No unique storage of this type exist.
    MissingUnique(&'static str),
    /// The unique storage is already borrowed.
    StorageBorrow((&'static str, Borrow)),
}

#[cfg(feature = "std")]
impl Error for UniqueRemove {}

impl Debug for UniqueRemove {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        match self {
            UniqueRemove::AllStorages => f.write_str("Cannot borrow AllStorages while it's already exclusively borrowed."),
            UniqueRemove::MissingUnique(name) => f.write_fmt(format_args!("No unique storage exists for {}.\n", name)),
            UniqueRemove::StorageBorrow((name, borrow)) => match borrow {
                Borrow::Unique => f.write_fmt(format_args!("Cannot mutably borrow {} storage while it's already borrowed.", name)),
                Borrow::WrongThread => f.write_fmt(format_args!("Cannot borrow {} storage from other thread than the one it was created in because it's !Send and !Sync.", name)),
                _ => unreachable!()
            }
        }
    }
}

impl Display for UniqueRemove {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        Debug::fmt(self, f)
    }
}

/// Error returned by [`apply`] and [`apply_mut`].
///
/// [`apply`]: crate::ViewMut::apply()
/// [`apply_mut`]: crate::ViewMut::apply_mut()
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Apply {
    #[allow(missing_docs)]
    IdenticalIds,
    /// Entity that doesn't have the required component.
    MissingComponent(EntityId),
}

#[cfg(feature = "std")]
impl Error for Apply {}

impl Debug for Apply {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        match self {
            Apply::IdenticalIds => f.write_str("Cannot use apply with identical components."),
            Apply::MissingComponent(id) => f.write_fmt(format_args!(
                "Entity {:?} does not have any component in this storage.",
                id
            )),
        }
    }
}

impl Display for Apply {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        Debug::fmt(self, f)
    }
}

/// Returned when trying to create views for custom storages.
pub enum CustomStorageView {
    #[allow(missing_docs)]
    GetStorage(GetStorage),
    #[allow(missing_docs)]
    WrongType(Cow<'static, str>),
}

impl From<GetStorage> for CustomStorageView {
    fn from(get_storage: GetStorage) -> CustomStorageView {
        CustomStorageView::GetStorage(get_storage)
    }
}

#[cfg(feature = "std")]
impl Error for CustomStorageView {}

impl Debug for CustomStorageView {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        match self {
            CustomStorageView::GetStorage(get_storage) => Debug::fmt(get_storage, f),
            CustomStorageView::WrongType(name) => f.write_fmt(format_args!(
                "Cannot convert, custom storage is of type: {:?}",
                name
            )),
        }
    }
}

impl Display for CustomStorageView {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        Debug::fmt(self, f)
    }
}

/// Returned by [`World::get`] and [`AllStorages::get`].
///
/// [`World::get`]: crate::World::get
/// [`AllStorages::get`]: crate::AllStorages::get
#[derive(PartialEq)]
pub enum GetComponent {
    #[allow(missing_docs)]
    StorageBorrow(GetStorage),
    #[allow(missing_docs)]
    MissingComponent(MissingComponent),
}

impl From<GetStorage> for GetComponent {
    fn from(get_storage: GetStorage) -> GetComponent {
        GetComponent::StorageBorrow(get_storage)
    }
}

impl From<MissingComponent> for GetComponent {
    fn from(missing_component: MissingComponent) -> GetComponent {
        GetComponent::MissingComponent(missing_component)
    }
}

#[cfg(feature = "std")]
impl Error for GetComponent {}

impl Debug for GetComponent {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        match self {
            GetComponent::StorageBorrow(err) => f.write_fmt(format_args!("{:?}", err)),
            GetComponent::MissingComponent(err) => f.write_fmt(format_args!("{:?}", err)),
        }
    }
}

impl Display for GetComponent {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        Debug::fmt(self, f)
    }
}
