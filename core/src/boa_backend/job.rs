use alloc::rc::Rc;
use core::cell::Cell;

use boa_engine::{
    job::{Job, JobExecutor, SimpleJobExecutor},
    Context as BoaContext, JsResult,
};

#[derive(Debug, Default)]
pub(crate) struct CompatJobExecutor {
    inner: Rc<SimpleJobExecutor>,
    pending_hint: Cell<usize>,
    executed_last_run: Cell<bool>,
}

impl CompatJobExecutor {
    pub(crate) fn new() -> Rc<Self> {
        Rc::new(Self {
            inner: Rc::new(SimpleJobExecutor::new()),
            pending_hint: Cell::new(0),
            executed_last_run: Cell::new(false),
        })
    }

    pub(crate) fn take_last_run_executed(&self) -> bool {
        self.executed_last_run.replace(false)
    }
}

impl JobExecutor for CompatJobExecutor {
    fn enqueue_job(self: Rc<Self>, job: Job, context: &mut BoaContext) {
        self.pending_hint
            .set(self.pending_hint.get().saturating_add(1));
        self.inner.clone().enqueue_job(job, context);
    }

    fn run_jobs(self: Rc<Self>, context: &mut BoaContext) -> JsResult<()> {
        let had_pending = self.pending_hint.get() > 0;
        self.executed_last_run.set(false);
        self.pending_hint.set(0);
        let result = self.inner.clone().run_jobs(context);
        self.executed_last_run.set(had_pending);
        self.pending_hint.set(0);
        result
    }
}
