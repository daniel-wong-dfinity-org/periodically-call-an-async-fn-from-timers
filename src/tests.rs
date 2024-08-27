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
    fn set_timer(&self, _delay: Duration, work: Box<dyn FnOnce () -> ()>) {
        work()
    }
}

#[test]
fn test_do_thing_repeatedly_in_the_background() {
    do_thing_repeatedly_in_the_background(FakeSetTimer {}, St {});
}
