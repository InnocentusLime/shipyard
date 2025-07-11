#[cfg(all(feature = "std", feature = "proc"))]
mod book;
mod borrow;
#[cfg(feature = "proc")]
mod derive;
mod iteration;
#[cfg(feature = "serde1")]
mod serde;

use std::iter::Sum;

use shipyard::*;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
struct USIZE(usize);
impl Component for USIZE {
    type Tracking = track::Untracked;
}

impl Sum for USIZE {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        USIZE(iter.map(|i| i.0).sum())
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
struct U32(u32);
impl Component for U32 {
    type Tracking = track::Untracked;
}

#[test]
fn run() {
    let world = World::new();
    world.run(
        |(mut entities, mut usizes, mut u32s): (EntitiesViewMut, ViewMut<USIZE>, ViewMut<U32>)| {
            entities.add_entity((&mut usizes, &mut u32s), (USIZE(0), U32(1)));
            entities.add_entity((&mut usizes, &mut u32s), (USIZE(2), U32(3)));

            // possible to borrow twice as immutable
            let mut iter1 = (&usizes).iter();
            let _iter2 = (&usizes).iter();
            assert_eq!(iter1.next(), Some(&USIZE(0)));

            // impossible to borrow twice as mutable
            // if switched, the next two lines should trigger an shipyard::error
            let _iter = (&mut usizes).iter();
            let mut iter = (&mut usizes).iter();
            assert_eq!(iter.next().map(|x| *x), Some(USIZE(0)));
            assert_eq!(iter.next().map(|x| *x), Some(USIZE(2)));
            assert!(iter.next().is_none());

            // possible to borrow twice as immutable
            let mut iter = (&usizes, &u32s).iter();
            let _iter = (&usizes, &u32s).iter();
            assert_eq!(iter.next(), Some((&USIZE(0), &U32(1))));
            assert_eq!(iter.next(), Some((&USIZE(2), &U32(3))));
            assert_eq!(iter.next(), None);

            // impossible to borrow twice as mutable
            // if switched, the next two lines should trigger an shipyard::error
            let _iter = (&mut usizes, &u32s).iter();
            let mut iter = (&mut usizes, &u32s).iter();
            assert_eq!(iter.next().map(|(x, y)| (*x, *y)), Some((USIZE(0), U32(1))));
            assert_eq!(iter.next().map(|(x, y)| (*x, *y)), Some((USIZE(2), U32(3))));
            assert!(iter.next().is_none());
        },
    );
}

#[test]
#[should_panic(expected = "Entity has to be alive to add component to it.")]
fn add_component_with_old_key() {
    let world = World::new();

    let entity = {
        let (mut entities, mut usizes, mut u32s) = world
            .borrow::<(EntitiesViewMut, ViewMut<USIZE>, ViewMut<U32>)>()
            .unwrap();
        entities.add_entity((&mut usizes, &mut u32s), (USIZE(0), U32(1)))
    };

    world.run(|mut all_storages: AllStoragesViewMut| {
        all_storages.delete_entity(entity);
    });

    let (entities, mut usizes, mut u32s) = world
        .borrow::<(EntitiesViewMut, ViewMut<USIZE>, ViewMut<U32>)>()
        .unwrap();

    entities.add_component(entity, (&mut usizes, &mut u32s), (USIZE(1), U32(2)));
}

#[test]
fn contains() {
    let world = World::new();

    world.run(
        |mut entities: EntitiesViewMut, mut usizes: ViewMut<USIZE>, mut u32s: ViewMut<U32>| {
            let entity = entities.add_entity((), ());

            entities.add_component(entity, &mut usizes, USIZE(0));

            assert!(usizes.contains(entity));
            assert!(!(&usizes, &u32s).contains(entity));

            entities.add_component(entity, &mut u32s, U32(1));

            assert!((&usizes, &u32s).contains(entity));
        },
    );
}

#[test]
fn debug() {
    let mut world = World::new();

    world.add_entity((USIZE(0),));
    world.add_entity((USIZE(1),));
    world.add_entity((USIZE(2),));

    world.run(|usizes: View<USIZE>| {
        assert_eq!(
            format!("{:?}", usizes),
            "[(EId(0.0), USIZE(0)), (EId(1.0), USIZE(1)), (EId(2.0), USIZE(2))]"
        );
    });
}
