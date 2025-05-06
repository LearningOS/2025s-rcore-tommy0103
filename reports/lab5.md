### sys_enable_deadlock_detect

模块化实现，抽象了一个新类型 `DeadlockChecker`，提供 `activate(), add_lock(), add_task(), try_lock(), lock(), unlock()` 五个接口，在对应进程，mutex / semaphore 创建时进行调用，由于 `Mutex` 和 `Semaphore` 在资源的本质上相同，所以可以使用该类型来作为 `mutex_checker` 和 `semaphore_checker` 的实例。 