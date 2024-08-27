use super::*;
use candid_parser::{
    utils::{
        CandidSource,
        service_equal,
    },
};


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
struct FakeSetTimer {}

impl SetTimer for FakeSetTimer {
    fn set_timer(&self, _delay: Duration, work: Box<dyn Send + FnOnce () -> ()>) {
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(50));
            work();
        });
    }
}

#[test]
fn test_do_thing_repeatedly_in_the_background() {
    // There are at least a couple (surmountable?) problems with this test:
    //
    //     1. do_thing_repeatedly_in_the_background calls ic_cdk::spawn, which
    //        ofc, does not work in unit tests. This can probably be easily
    //        overcome by introducing a Spawn trait.
    //
    //     2. There is no way to see the effect of St. If we pass it a reference
    //        to something, we could inspect that object later. E.g.
    //        thread_local! { static M: RefCell<...> = ...; }
    do_thing_repeatedly_in_the_background(FakeSetTimer {}, St {});
}
