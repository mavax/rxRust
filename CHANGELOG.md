## [Unreleased](https://github.com/M-Adoo/rxRust/compare/v0.3.0...HEAD)

## [0.4.0](https://github.com/M-Adoo/rxRust/releases/tag/v0.4.0)  (2019-11-07)

### Features
- **observable**: add `ConnectableObservable` to support multicast.
- **operator**: add `throttle_time` operator
- **operator**: add `publish` operator
- **operator**: add `ref_count` operator
- **Subject**: support `Fork` even if `Item` and `Err` not support `Clone`.

### Breaking Changes

**Scheduler**: add a `delay` param for `schedule` method, from 
```
pub trait Scheduler {
  fn schedule<T: Send + Sync + 'static>(
    &self,
    task: impl FnOnce(SharedSubscription, T) + Send + 'static,
    state: T,
  ) -> SharedSubscription;
}
```
to
```
pub trait Scheduler {
  fn schedule<T: Send + 'static>(
    &self,
    task: impl FnOnce(SharedSubscription, T) + Send + 'static,
    delay: Option<Duration>,
    state: T,
  ) -> SharedSubscription;
}
```

## [0.3.0](https://github.com/M-Adoo/rxRust/releases/tag/v0.3.0)  (2019-10-12)

### Code Refactoring

In `v0.2` we implemented all operators and observable thread safe， so we can pass task across threads by schedulers. In this way, all user provide closure must satisfied `Send + Sync + 'static`, even never use scheduler and multi-thread.

For now, we removed the bounds `Sync`, `Send` and `'static`, and add a new trait `IntoShared`. We always implemented operator for local thread, and implement `IntoShared` for it to convert it to a thread-safe operator.
By default, RxRust always use single thread version to get the best performance, and use `IntoShared` to convert a local object to a thread-safe object if we need pass this object in threads.

**Before**:
```rust
let res = Arc::new(Mutex(0));w
let c_res = res.clone();
observable::of(100).subscribe(|v| { *res.lock().unwrap() = *v });

assert_eq!(*res.lock().unwrap(), 100);
```

**After**:

```rust
let mut res = 0;
observable::of(100).subscribe(|v| { res = *v });

assert_eq!(res, 100);
```

### Breaking Changes

- removed `RxFn` and `RxValue`
- **operators**: removed  `Multicast`
- **observable**: removed `ObservableOnce`
- **observable**: `observable::from_vec` and `observable::from_range` functions merge to `observable::from_iter!` macro.
- **observable**: `observable::empty` function  to `observable::empty!` macro.
- **observable**: `observable::of` function to `observable::of!` macro.
- **observable**: `observable::from_future` function to `observable::from_future!` macro
- **observable**: `observable::from_future_with_err` function to `observable::from_future_with_err!` macro
- **observable**: `observable::interval` function to `observable::interval!` macro

### Bug Fixes

- **observe_on**: unsubscribe should also cancel dispatched message.
- **subscribe_on**: unsubscribe should also cancel task in scheduler queue.

## [0.2.0](https://github.com/M-Adoo/rxRust/releases/tag/v0.2.0)  (2019-09-02)

### Features
- **observable**: add `observable::from_vec` and `observable::from_range`
- **observable**: add `observable::empty` and `observable::of`
- **observable**: add `observable::from_future` and `observable::from_future_with_err`
- **observable**: add `observable::interval`
- **operator**: add `delay` operator 
- **operator**: add `filter` operator 
- **operator**: add `first` operator 
- **operator**: add `multicast` and `fork` operator, `multicast` and `fork` are  special operators in rxrust, that because in rxrust all operators both consume the upstream, so the are unicast, `multicast` let you can convert an unicast stream to a multicast stream to support `fork` stream from it.
- **operator**: add `map` operator 
- **operator**: add `merge` operator
- **operator**: add `observe_on` operator
- **operator**: add `subscribe_on` operator
- **operator**: add `take` operator
- **Schedulers**: add `Schedulers::Sync` implementation
- **Schedulers**: add `Schedulers::NewThread` implementation
- **Schedulers**: add `Schedulers::ThreadPool` implementation