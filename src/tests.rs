use super::*;
use candid_parser::{
    utils::{
        CandidSource,
        service_equal,
    },
};
use std::sync::{Arc, Mutex, Condvar};


#[test]
fn test_implemented_interface_matches_declared_interface_exactly() {
    let declared_interface = CandidSource::Text(include_str!("../my-canister.did"));

    // The line below generates did types and service definition from the
    // methods annotated with `candid_method` above. The definition is then
    // obtained with `__export_service()`.
    candid::export_service!();
    let implemented_interface_str = __export_service();
    let implemented_interface = CandidSource::Text(&implemented_interface_str);

    let result = service_equal(declared_interface, implemented_interface);
    assert!(result.is_ok(), "{:?}\n\n", result.unwrap_err());
}

#[derive(Clone)]
struct FakeCanisterHost {
    is_done: Arc<(Mutex<bool>, Condvar)>,
    stop: Arc<(Mutex<bool>, Condvar)>,
}

impl CanisterHost for FakeCanisterHost {
    fn set_timer(&self, _delay: Duration, work: Box<dyn Send + FnOnce () -> ()>) {
        let is_done = Arc::clone(&self.is_done);
        self.spawn(async move {
            work();

            println!("Work is done.");

            {
                let (mutex, cond_var) = &*is_done;
                let mut done = mutex.lock().unwrap();
                *done = true;
                cond_var.notify_one();
            }
        });
    }

    fn spawn<FutureImpl>(&self, future: FutureImpl)
    where
        FutureImpl: Future<Output = ()> + Send + 'static
    {
        {
            let (mutex, _) = &*self.stop;
            let stop = mutex.lock().unwrap();
            if *stop {
                return;
            }
        }

        tokio::task::spawn(future);
    }
}

#[test]
fn test_do_thing_repeatedly_in_the_background() {
    let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();

    runtime.block_on(async {
        let is_done = Arc::new((Mutex::new(false), Condvar::new()));
        let wait_for_is_done = Arc::clone(&is_done);

        let stop = Arc::new((Mutex::new(false), Condvar::new()));
        let send_stop = Arc::clone(&stop);

        // There are at least a couple (surmountable?) problems with this test:
        //
        //     1. There is no way to see the effect of GreeterImpl. If we pass
        //        it a reference to something, we could inspect that object
        //        later. E.g. thread_local! { static M: RefCell<...> = ...; }
        do_thing_repeatedly_in_the_background(FakeCanisterHost { is_done, stop }, GreeterImpl {});

        {
            let (mutex, cond_var) = &*wait_for_is_done;
            let mut is_done = mutex.lock().unwrap();
            while !*is_done {
                is_done = cond_var.wait(is_done).unwrap();
            }
        }

        {
            let (mutex, cond_var) = &*send_stop;
            let mut stop = mutex.lock().unwrap();
            *stop = true;
            cond_var.notify_one();
        }

        println!("END");
    });
}
