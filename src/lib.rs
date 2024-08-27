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
trait Tr {
    async fn howdy(&self);
}

struct St {}

#[async_trait]
impl Tr for St {
    async fn howdy(&self) {
        let (config,) = storage::stable_restore::<(Config,)>()
            .expect("Unable to retrieve Config from stable memory.");

        println!("\nConfig: {:#?}\n", config);
    }
}

trait SetTimer {
    fn set_timer(&self, delay: Duration, work: Box<dyn FnOnce () -> ()>);
}

#[derive(Clone)]
struct IcCdkSetTimer {}

impl SetTimer for IcCdkSetTimer {
    fn set_timer(&self, delay: Duration, work: Box<dyn FnOnce () -> ()>) {
        ic_cdk_timers::set_timer(delay, work);
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

    do_thing_repeatedly_in_the_background(IcCdkSetTimer {}, St {});
}

fn do_thing_repeatedly_in_the_background<SetTimerImpl, TrImpl>(
    set_timer: SetTimerImpl, tr: TrImpl)
where
    SetTimerImpl: SetTimer + Clone + 'static,
    TrImpl: Tr + 'static,
{
    let next_set_timer = set_timer.clone();

    set_timer.set_timer(
        Duration::from_millis(500),
        Box::new(move || {
            ic_cdk::spawn(async {
                tr.howdy().await;
                do_thing_repeatedly_in_the_background(next_set_timer, tr);
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
