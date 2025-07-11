use crate::error;
use crate::scheduler::{Batches, Label};
use crate::world::World;
use alloc::boxed::Box;

impl World {
    #[allow(clippy::type_complexity)]
    pub(crate) fn run_batches_sequential(
        &self,
        systems: &[Box<dyn Fn(&World) -> Result<(), error::Run> + Send + Sync + 'static>],
        system_names: &[Box<dyn Label>],
        batches: &Batches,
        #[cfg_attr(not(feature = "tracing"), allow(unused))] workload_name: &dyn Label,
    ) -> Result<(), error::RunWorkload> {
        #[cfg(feature = "tracing")]
        let parent_span = tracing::info_span!("workload", name = ?workload_name);
        #[cfg(feature = "tracing")]
        let _parent_span = parent_span.enter();

        batches
            .sequential
            .iter()
            .zip(&batches.sequential_run_if)
            .try_for_each(|(&index, run_if)| {
                if let Some(run_if) = run_if.as_ref() {
                    let should_run = (run_if)(self).map_err(|err| {
                        error::RunWorkload::Run((system_names[index].clone(), err))
                    })?;

                    if !should_run {
                        return Ok(());
                    }
                }

                #[cfg(feature = "tracing")]
                {
                    self.run_single_system(systems, system_names, &parent_span, index)
                }
                #[cfg(not(feature = "tracing"))]
                {
                    self.run_single_system(systems, system_names, index)
                }
            })
    }

    #[allow(clippy::type_complexity)]
    fn run_single_system(
        &self,
        systems: &[Box<dyn Fn(&World) -> Result<(), error::Run> + Send + Sync>],
        system_names: &[Box<dyn Label>],
        #[cfg(feature = "tracing")] parent_span: &tracing::Span,
        index: usize,
    ) -> Result<(), error::RunWorkload> {
        #[cfg(feature = "tracing")]
        let system_span =
            tracing::info_span!(parent: parent_span.clone(), "system", name = ?system_names[index]);
        #[cfg(feature = "tracing")]
        let _system_span = system_span.enter();

        (systems[index])(self)
            .map_err(|err| error::RunWorkload::Run((system_names[index].clone(), err)))
    }
}
