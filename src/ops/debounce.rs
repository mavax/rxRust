use crate::prelude::*;
use observable::observable_proxy_impl;
use std::{
  cell::RefCell,
  rc::Rc,
  sync::{Arc, Mutex},
  time::{Duration, Instant},
};

#[derive(Clone)]
pub struct DebounceOp<S, SD> {
  pub(crate) source: S,
  pub(crate) scheduler: SD,
  pub(crate) duration: Duration,
}

observable_proxy_impl!(DebounceOp, S, SD);

impl<Item, Err, S, SD, Unsub> LocalObservable<'static> for DebounceOp<S, SD>
where
  S: LocalObservable<'static, Item = Item, Err = Err, Unsub = Unsub>,
  Unsub: SubscriptionLike + 'static,
  Item: Clone + 'static,
  SD: LocalScheduler + 'static,
{
  type Unsub = Unsub;

  fn actual_subscribe<O: Observer<Self::Item, Self::Err> + 'static>(
    self,
    subscriber: Subscriber<O, LocalSubscription>,
  ) -> Self::Unsub {
    let Self {
      source,
      scheduler,
      duration,
    } = self;

    source.actual_subscribe(Subscriber {
      observer: LocalDebounceObserver(Rc::new(RefCell::new(
        DebounceObserver {
          observer: subscriber.observer,
          delay: duration,
          scheduler,
          trailing_value: None,
          last_updated: None,
        },
      ))),
      subscription: subscriber.subscription,
    })
  }
}
impl<S, SD> SharedObservable for DebounceOp<S, SD>
where
  S: SharedObservable,
  S::Item: Clone + Send + 'static,
  SD: SharedScheduler + Send + 'static,
{
  type Unsub = S::Unsub;
  fn actual_subscribe<
    O: Observer<Self::Item, Self::Err> + Sync + Send + 'static,
  >(
    self,
    subscriber: Subscriber<O, SharedSubscription>,
  ) -> S::Unsub {
    let Self {
      source,
      duration,
      scheduler,
    } = self;
    let Subscriber {
      observer,
      subscription,
    } = subscriber;
    source.actual_subscribe(Subscriber {
      observer: SharedDebounceObserver(Arc::new(Mutex::new(
        DebounceObserver {
          observer,
          scheduler,
          trailing_value: None,
          delay: duration,
          last_updated: None,
        },
      ))),
      subscription,
    })
  }
}

struct DebounceObserver<O, S, Item> {
  observer: O,
  scheduler: S,
  delay: Duration,
  trailing_value: Option<Item>,
  last_updated: Option<Instant>,
}

struct SharedDebounceObserver<O, S, Item>(
  Arc<Mutex<DebounceObserver<O, S, Item>>>,
);

struct LocalDebounceObserver<O, S, Item>(
  Rc<RefCell<DebounceObserver<O, S, Item>>>,
);

macro impl_debounce_observer($item:ident, $err:ident) {
  fn next(&mut self, value: $item) {
    let mut c_inner = self.0.clone();
    let mut inner = self.0.inner_deref_mut();
    let updated = Some(Instant::now());
    inner.last_updated = updated.clone();
    inner.trailing_value = Some(value.clone());
    let delay = inner.delay;
    inner.scheduler.schedule(
      move |last| {
        let mut inner = c_inner.inner_deref_mut();
        if let Some(value) = inner.trailing_value.clone() {
          if inner.last_updated == last {
            inner.observer.next(value);
            inner.trailing_value = None;
          }
        }
      },
      Some(delay),
      inner.last_updated.clone(),
    );
  }
  fn error(&mut self, err: $err) {
    let mut inner = self.0.inner_deref_mut();
    inner.observer.error(err)
  }
  fn complete(&mut self) {
    let mut inner = self.0.inner_deref_mut();
    if let Some(value) = inner.trailing_value.take() {
      inner.observer.next(value);
    }
    inner.observer.complete();
  }
  fn is_stopped(&self) -> bool {
    let inner = self.0.inner_deref();
    inner.observer.is_stopped()
  }
}

impl<O, S, Item, Err> Observer<Item, Err> for SharedDebounceObserver<O, S, Item>
where
  O: Observer<Item, Err> + Send + 'static,
  S: SharedScheduler + Send + 'static,
  Item: Clone + Send + 'static,
{
  impl_debounce_observer!(Item, Err);
}

impl<O, S, Item, Err> Observer<Item, Err> for LocalDebounceObserver<O, S, Item>
where
  O: Observer<Item, Err> + 'static,
  S: LocalScheduler + 'static,
  Item: Clone + 'static,
{
  impl_debounce_observer!(Item, Err);
}

#[cfg(test)]
mod tests {
  use super::*;
  use futures::executor::LocalPool;
  #[test]
  fn smoke_last() {
    let x = Rc::new(RefCell::new(vec![]));
    let x_c = x.clone();
    let mut pool = LocalPool::new();
    let interval =
      observable::interval(Duration::from_millis(2), pool.spawner());
    let spawner = pool.spawner();
    let debounce_subscribe = || {
      let x = x.clone();
      interval
        .clone()
        .take(10)
        .debounce(Duration::from_millis(3), spawner.clone())
        .subscribe(move |v| x.borrow_mut().push(v))
    };
    let mut sub = debounce_subscribe();
    pool.run();
    sub.unsubscribe();
    assert_eq!(&*x_c.borrow(), &[9]);
  }

  #[test]
  fn smoke_every() {
    let x = Rc::new(RefCell::new(vec![]));
    let x_c = x.clone();
    let mut pool = LocalPool::new();
    let interval =
      observable::interval(Duration::from_millis(3), pool.spawner());
    let spawner = pool.spawner();
    let debounce_subscribe = || {
      let x = x.clone();
      interval
        .clone()
        .take(10)
        .debounce(Duration::from_millis(2), spawner.clone())
        .subscribe(move |v| x.borrow_mut().push(v))
    };
    let mut sub = debounce_subscribe();
    pool.run();
    sub.unsubscribe();
    assert_eq!(&*x_c.borrow(), &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
  }

  #[test]
  fn fork_and_shared() {
    use futures::executor::ThreadPool;
    let scheduler = ThreadPool::new().unwrap();
    observable::from_iter(0..10)
      .debounce(Duration::from_nanos(1), scheduler)
      .to_shared()
      .to_shared()
      .subscribe(|_| {});
  }
}
