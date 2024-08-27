use async_trait::async_trait;
use ic_cdk::{
    export::{
        candid::{
            CandidType,
            Deserialize,
        },
    },
    storage,
};
use ic_cdk_macros::{init, post_upgrade, query};
use std::{
    fmt::{
        Debug,
    },
    future::{
        Future,
    },
    time::{
        Duration,
    },
};

#[cfg(target_arch = "wasm32")]
use ic_cdk::println;

#[derive(CandidType, Deserialize, Clone, Debug, Default)]
struct Config {
    message: String,
}

#[async_trait]
trait Greeter {
    async fn howdy(&self);
}

struct GreeterImpl {}

#[async_trait]
impl Greeter for GreeterImpl {
    async fn howdy(&self) {
        println!("Ahoy!");
    }
}

trait CanisterHost {
    fn set_timer(&self, delay: Duration, work: Box<dyn Send + FnOnce () -> ()>);
    fn spawn<FutureImpl>(&self, future: FutureImpl)
        where FutureImpl: Future<Output = ()> + Send + 'static;
}

#[derive(Clone)]
struct RealCanisterHost {}

impl CanisterHost for RealCanisterHost {
    fn set_timer(&self, delay: Duration, work: Box<dyn Send + FnOnce () -> ()>) {
        ic_cdk_timers::set_timer(delay, work);
    }

    fn spawn<FutureImpl>(&self, future: FutureImpl)
    where
        FutureImpl: Future<Output = ()> + Send + 'static
    {
        ic_cdk::spawn(future);
    }
}

#[init]
fn init(config: Config) {
    storage::stable_save((config,)).expect("Failed to save config to stable storage");
}

#[post_upgrade]
fn post_upgrade(config: Option<Config>) {
    let config = config.unwrap();
    storage::stable_save((config,)).expect("Failed to save config to stable storage");

    do_thing_repeatedly_in_the_background(RealCanisterHost {}, GreeterImpl {});
}

fn do_thing_repeatedly_in_the_background<CanisterHostImpl, GreeterImpl>(
    canister_host: CanisterHostImpl, greeter: GreeterImpl)
where
    CanisterHostImpl: CanisterHost + Send + Clone + 'static,
    GreeterImpl: Greeter + Send + 'static,
{
    canister_host.clone().set_timer(
        Duration::from_millis(500),
        Box::new(move || {
            canister_host.clone().spawn(async {
                greeter.howdy().await;
                do_thing_repeatedly_in_the_background(canister_host, greeter);
            })
        }),
    );
}

#[query]
fn get_config() -> Config {
    storage::stable_restore::<(Config,)>().expect("Failed to restore config from stable storage").0
}

// This must occur at the end. Please, do not move to the top (where we usually put mods).
#[cfg(test)]
mod tests;
