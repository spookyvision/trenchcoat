// adapted from https://gist.github.com/Demonthos/74301fde7f6120d1b1b06adc7e6c40ea

#![allow(non_snake_case)]
use std::{
    cell::{Ref, RefCell, RefMut},
    collections::HashSet,
    fmt::Display,
    rc::Rc,
    sync::Mutex,
};

use dioxus::prelude::*;

#[derive(Clone)]
pub struct SplitSubscription<S, T>
where
    S: Clone,
    T: Clone,
{
    pub state: LocalSubscription<S>,
    pub t: Rc<Mutex<T>>,
}

impl<S, T> SplitSubscription<S, T>
where
    S: Clone,
    T: Clone,
{
    pub fn new(state: LocalSubscription<S>, t: T) -> Self {
        Self {
            state,
            t: Rc::new(Mutex::new(t)),
        }
    }
}

pub fn use_local_subscription_root<S, T>(cx: &ScopeState) -> &SplitSubscription<S, T>
where
    S: Default + Clone + 'static,
    T: Default + Clone + 'static,
{
    use_context_provider(cx, || {
        let state = LocalSubscription::create(cx, Default::default());
        let t = Default::default();
        SplitSubscription { state, t }
    })
}

pub fn use_split_subscriptions<S, T>(cx: &ScopeState) -> &SplitSubscription<S, T>
where
    S: Default + Clone + 'static,
    T: Default + Clone + 'static,
{
    use_context(cx).expect("No SplitSubscription found")
}

#[derive(Clone)]
pub struct LocalSubscription<T> {
    inner: Rc<RefCell<T>>,
    subscribed: Rc<RefCell<HashSet<ScopeId>>>,
    update: Rc<dyn Fn()>,
}

impl<T: 'static> LocalSubscription<T> {
    pub fn create(cx: &ScopeState, inner: T) -> Self {
        let inner = Rc::new(RefCell::new(inner));
        let update_any = cx.schedule_update_any();
        let subscribed: Rc<RefCell<HashSet<ScopeId>>> = Default::default();
        let update = Rc::new({
            to_owned![subscribed];
            move || {
                for id in subscribed.borrow().iter() {
                    update_any(*id);
                }
            }
        });
        Self {
            inner,
            subscribed,
            update,
        }
    }

    pub fn use_state<'a>(&self, cx: &'a ScopeState) -> &'a UseLocal<T> {
        cx.use_hook(|| {
            let id = cx.scope_id();
            self.subscribed.borrow_mut().insert(id);

            UseLocal {
                inner: self.inner.clone(),
                update: self.update.clone(),
            }
        })
    }

    pub fn use_write_only<'a>(&self, cx: &'a ScopeState) -> &'a LocalWrite<T> {
        cx.use_hook(|| LocalWrite {
            inner: self.inner.clone(),
            update: self.update.clone(),
        })
    }

    // This should only be used outside of components. This will not subscribe to any state.
    pub fn write(&self) -> RefMut<T> {
        (self.update)();
        self.inner.borrow_mut()
    }

    // This should only be used outside of components. This will not subscribe to any state.
    pub fn read(&self) -> Ref<T> {
        self.inner.borrow()
    }
}

/// A read/write version of the local state. This allows mutating the state and reading state.
pub struct UseLocal<T> {
    inner: Rc<RefCell<T>>,
    update: Rc<dyn Fn()>,
}

impl<T> UseLocal<T> {
    pub fn read(&self) -> Ref<T> {
        self.inner.borrow()
    }

    pub fn write(&self) -> RefMut<T> {
        (self.update)();
        self.inner.borrow_mut()
    }
}

impl<T: Display> Display for UseLocal<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.borrow().fmt(f)
    }
}

/// A write only version of the local state. This only allows mutating the state, not reading state because you can only access the inner type in a impl Fn(&mut T) closure.
pub struct LocalWrite<T> {
    inner: Rc<RefCell<T>>,
    update: Rc<dyn Fn()>,
}

impl<T> LocalWrite<T> {
    pub fn with_mut(&self, f: impl Fn(&mut T)) {
        f(&mut *self.inner.borrow_mut());
        (self.update)();
    }
}
