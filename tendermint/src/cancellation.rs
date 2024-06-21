use std::{
    marker::PhantomData,
    rc::Rc,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock,
    },
};

pub const CANCEL_PANIC_MSG: &'static str = "requested cancellation from other thread";

static FLAG: AtomicBool = AtomicBool::new(false);

pub type EmptyCtx = ();

#[derive(Debug, Clone, Default)]
pub enum CancellationContext<T: Clone + Send + Sync> {
    #[default]
    None,
    GasOverflow,
    Other(T),
}

#[derive(Debug, Clone)]
pub struct CancellationToken<T: Clone + Send + Sync>(Arc<RwLock<CancellationContext<T>>>);

impl<T: Clone + Send + Sync> CancellationToken<T> {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(CancellationContext::None)))
    }

    pub fn new_with(ctx: T) -> Self {
        Self(Arc::new(RwLock::new(CancellationContext::Other(ctx))))
    }

    pub fn is_cancelled(&self) -> bool {
        FLAG.load(Ordering::Relaxed)
    }

    pub fn if_cancelled_ctx(&self) -> Option<CancellationContext<T>> {
        match self.is_cancelled() {
            true => Some(self.0.read().expect("poisoned lock").to_owned()),
            false => None,
        }
    }

    pub fn panic_if_cancelled(&self) {
        panic!("{CANCEL_PANIC_MSG}")
    }

    pub fn cancel(&self) {
        FLAG.store(true, Ordering::Relaxed)
    }

    pub fn cancel_with_context(&self, ctx: CancellationContext<T>) {
        self.cancel();
        self.0.clear_poison();

        *self.0.write().expect("poisoned lock") = ctx;
    }

    pub fn drop_guard(&self) -> TokenDropGuard<T> {
        TokenDropGuard::new(self.clone())
    }

    pub fn cancel_and_panic(&self) -> ! {
        self.cancel();
        self.panic_if_cancelled();
        unreachable!()
    }

    pub fn cancel_and_panic_with_context(&self, ctx: CancellationContext<T>) -> ! {
        self.cancel_with_context(ctx);
        self.panic_if_cancelled();
        unreachable!()
    }
}

#[derive(Debug)]
pub struct TokenDropGuard<T: Clone + Send + Sync> {
    flag: bool,
    token: CancellationToken<T>,
    /// I don't want that someone have ability to send token to other thread or save it to use somewhere else.
    /// So this marker prevents user from doing so by making type !Send & !Sync
    _marker: PhantomData<Rc<()>>,
}

impl<T: Clone + Send + Sync> TokenDropGuard<T> {
    fn new(token: CancellationToken<T>) -> Self {
        Self {
            flag: true,
            token,
            _marker: PhantomData,
        }
    }

    pub fn disarm(mut self) {
        self.flag = false;
    }
}

impl<T: Clone + Send + Sync> Drop for TokenDropGuard<T> {
    fn drop(&mut self) {
        // Other way is to catch panic and cancel in such case,
        // but I think it slightly unclear but cleaner
        if self.flag {
            self.token.cancel()
        }
    }
}
