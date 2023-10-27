#![allow(dead_code)]
use sai::port::{AutoNegConfigMode, Port};
use std::{
    fmt::Display,
    time::{Duration, SystemTime},
};

pub(super) trait State
where
    Self: std::fmt::Debug + Display + Copy,
{
}

pub(super) trait FromState<S: State>: Sized {
    fn from_state<'a>(from: Discovery<S>, port: &Port<'a>) -> Discovery<Self>
    where
        Self: State;
}

pub(super) trait IntoState<S: State>: Sized {
    fn into_state<'a>(self, port: &Port<'a>) -> Discovery<S>
    where
        S: State;
}

impl<S1: State, S2: State> IntoState<S2> for Discovery<S1>
where
    S2: FromState<S1>,
{
    fn into_state<'a>(self, port: &Port<'a>) -> Discovery<S2> {
        S2::from_state(self, port)
    }
}

pub(super) enum Event {
    NoChange,
    PortUp,
}

pub(super) enum DiscoveryStateMachine {
    Start(Discovery<Start>),
    AutoNeg(Discovery<AutoNeg>),
    Speed(Discovery<Speed>),
    Done(Discovery<Done>),
}

impl DiscoveryStateMachine {
    /// initializes a new logical port discovery state machine
    pub(super) fn new<'a>(
        port: &Port<'a>,
        supported_speeds: Vec<u32>,
        speed: u32,
        auto_negotiation: bool,
    ) -> Self {
        DiscoveryStateMachine::Start(Discovery::new(
            port,
            supported_speeds,
            speed,
            auto_negotiation,
        ))
    }

    pub(super) fn is_done(&self) -> bool {
        match self {
            DiscoveryStateMachine::Done(_) => true,
            _ => false,
        }
    }

    pub(super) fn is_done_and_success(&self) -> bool {
        match self {
            DiscoveryStateMachine::Done(v) => v.state.success,
            _ => false,
        }
    }

    fn can_step(&self) -> bool {
        match self {
            DiscoveryStateMachine::Start(s) => s.can_step(),
            DiscoveryStateMachine::AutoNeg(s) => s.can_step(),
            DiscoveryStateMachine::Speed(s) => s.can_step(),
            DiscoveryStateMachine::Done(s) => s.can_step(),
        }
    }

    pub(super) fn step<'a>(self, port: &Port<'a>, ev: Event) -> Self {
        match ev {
            Event::PortUp => match self {
                // as soon as we get a port up event, we simply consider this a success
                // regardless from which state we came
                // there is also no actions that need to be performed any longer
                // TODO: this is pretty verbose, but I don't see how this could be simplified
                DiscoveryStateMachine::Start(v) => {
                    DiscoveryStateMachine::Done(Discovery::success(v, port))
                }
                DiscoveryStateMachine::AutoNeg(v) => {
                    DiscoveryStateMachine::Done(Discovery::success(v, port))
                }
                DiscoveryStateMachine::Speed(v) => {
                    DiscoveryStateMachine::Done(Discovery::success(v, port))
                }
                DiscoveryStateMachine::Done(v) => {
                    DiscoveryStateMachine::Done(Discovery::success(v, port))
                }
            },
            Event::NoChange => {
                // if there is no change (which means a port did not come up), then
                // we first check if we can already step, or if not enough time has passed yet
                if !self.can_step() {
                    return self;
                }

                // if we can step, then we perform the *actual* state transitions right now here
                match self {
                    DiscoveryStateMachine::Start(v) => {
                        DiscoveryStateMachine::Speed(v.into_state(port))
                    }
                    DiscoveryStateMachine::Speed(v) if v.state.index == 0 => {
                        DiscoveryStateMachine::AutoNeg(v.into_state(port))
                    }
                    DiscoveryStateMachine::Speed(v) => {
                        DiscoveryStateMachine::Speed(v.into_state(port))
                    }
                    DiscoveryStateMachine::AutoNeg(v) => {
                        DiscoveryStateMachine::Done(v.into_state(port))
                    }
                    DiscoveryStateMachine::Done(_) => self,
                }
            }
        }
    }
}

#[derive(Debug)]
pub(super) struct Discovery<S: State> {
    transition_ts: SystemTime,
    transition_time: Duration,
    speed: u32,
    auto_negotiation: bool,
    supported_speeds: Vec<u32>,
    state: S,
}

impl<S: State> Discovery<S> {
    fn can_step(&self) -> bool {
        self.transition_ts.elapsed().unwrap_or_default() > self.transition_time
    }
}

fn set_admin_state<'a, S: State, T: State>(
    from_state: S,
    to_state: T,
    port: &Port<'a>,
    admin_state: bool,
) {
    if let Err(e) = port.set_admin_state(admin_state) {
        log::error!(
            "Port {}: state machine: {} -> {}: failed to set admin state to {}: {}",
            port,
            from_state,
            to_state,
            admin_state,
            e
        );
    }
}

fn disable_auto_negotiation<'a, S: State, T: State>(
    from_state: S,
    to_state: T,
    port: &Port<'a>,
    auto_negotiation: bool,
) -> bool {
    if auto_negotiation {
        // auto negotiation advertisement might also be enabled then
        // let's just try to disable it as well, but no need to account for that for the state
        if let Err(e) = port.set_advertised_auto_neg_mode(false) {
            log::error!(
                "Port {} state machine: {} -> {}: failed to set auto negotiation advertisement to off: {}",
                port,
                from_state,
                to_state,
                e
            );
        }
        match port.set_auto_neg_mode(false) {
            Ok(_) => false,
            Err(e) => {
                log::error!(
                    "Port {} state machine: {} -> {}: failed to set auto negotiation to off: {}",
                    port,
                    from_state,
                    to_state,
                    e
                );
                true
            }
        }
    } else {
        false
    }
}

impl Discovery<Start> {
    /// Creates a new state machine for the given port. The current supported speeds, speed and auto negotiation settings
    /// of the port must be passed in. The state machine will reconfigure them if they are not at the right state for the
    /// beginning of the state machine.
    fn new<'a>(
        port: &Port<'a>,
        supported_speeds: Vec<u32>,
        speed: u32,
        auto_negotiation: bool,
    ) -> Self {
        let state = Start {};

        // we need to sort the supported speeds to make sure they are ordered from low to high
        // probably already the case, but this makes sure it really is
        let mut supported_speeds = supported_speeds;
        supported_speeds.sort();

        // ensure admin state is down first
        set_admin_state(state, state, port, false);

        // ensure auto negotiation is disabled
        let auto_negotiation = disable_auto_negotiation(state, state, port, auto_negotiation);

        // ensure speed is set to the highest supported speed
        let speed = supported_speeds
            .last()
            .map(|highest_speed| {
                if speed != *highest_speed {
                    match port.set_speed(*highest_speed) {
                        Ok(_) => *highest_speed,
                        Err(e) => {
                            log::error!(
                                "Port {} state machine: {}: failed to set speed to {}: {}",
                                port,
                                state,
                                highest_speed,
                                e
                            );
                            speed
                        }
                    }
                } else {
                    speed
                }
            })
            .unwrap_or(speed);

        // bring admin state up
        set_admin_state(state, state, port, true);

        Discovery {
            transition_ts: SystemTime::now(),
            transition_time: Duration::from_secs(5),
            auto_negotiation: auto_negotiation,
            speed: speed,
            supported_speeds: supported_speeds,
            state: state,
        }
    }
}

impl Discovery<Done> {
    fn success<'a, T: State>(from: Discovery<T>, port: &Port<'a>) -> Self {
        let ret = Discovery {
            transition_ts: SystemTime::now(),
            transition_time: Default::default(),
            speed: from.speed,
            auto_negotiation: from.auto_negotiation,
            supported_speeds: from.supported_speeds,
            state: Done { success: true },
        };
        log::info!(
            "Port {}: state machine: {} -> {}: successfully brought up port",
            port,
            from.state,
            ret.state
        );
        ret
    }
}

impl FromState<Start> for Speed {
    fn from_state<'a>(from: Discovery<Start>, port: &Port<'a>) -> Discovery<Self> {
        // we are going to work our way down from the highest supported speed
        let idx = from.supported_speeds.len();

        // if we have zero length, then the supported speeds were empty for some reason, nothing to do here then
        if idx == 0 {
            let state = Speed { index: 0 };
            log::warn!(
                "Port {}: state machine: {} -> {}: no supported speeds found",
                port,
                from.state,
                state
            );
            return Discovery {
                transition_ts: SystemTime::now(),
                transition_time: Default::default(),
                speed: from.speed,
                auto_negotiation: from.auto_negotiation,
                supported_speeds: from.supported_speeds,
                state: state,
            };
        }

        // get the speed that we are going to work with
        let state = Speed { index: idx - 1 };
        let speed = from.supported_speeds[state.index];

        // bring the port admin state down
        set_admin_state(from.state, state, port, false);

        match port.set_speed(speed) {
            Ok(_) => log::debug!(
                "Port {}: state machine: {} -> {}: set speed to {} successful",
                port,
                from.state,
                state,
                speed
            ),
            Err(e) => log::error!(
                "Port {}: state machine: {} -> {}: failed to set speed to {}: {}",
                port,
                from.state,
                state,
                speed,
                e
            ),
        }

        // and bring the admin state up again
        set_admin_state(from.state, state, port, true);

        // we are going to give a speed change up to 5 seconds to bring up a port
        Discovery {
            transition_ts: SystemTime::now(),
            transition_time: Duration::from_secs(5),
            speed: speed,
            auto_negotiation: from.auto_negotiation,
            supported_speeds: from.supported_speeds,
            state: state,
        }
    }
}

impl FromState<Speed> for Speed {
    fn from_state<'a>(from: Discovery<Speed>, port: &Port<'a>) -> Discovery<Self> {
        // check first if we are in an invalid state
        if from.state.index == 0 {
            log::error!(
                "Port {}: state machine: {} -> {}: already at lowest supported speed. State machine transition error!",
                port,
                from.state,
                from.state
            );
            return from;
        }

        // get the next speed that we need to try
        let state = Speed {
            index: from.state.index - 1,
        };
        let speed = from.supported_speeds[state.index];

        // bring the port admin state down
        set_admin_state(from.state, state, port, false);

        match port.set_speed(speed) {
            Ok(_) => log::debug!(
                "Port {}: state machine: {} -> {}: set speed to {} successful",
                port,
                from.state,
                state,
                speed
            ),
            Err(e) => log::error!(
                "Port {}: state machine: {} -> {}: failed to set speed to {}: {}",
                port,
                from.state,
                state,
                speed,
                e
            ),
        }

        // and bring the admin state up again
        set_admin_state(from.state, state, port, true);

        // we are going to give a speed change up to 5 seconds to bring up a port
        Discovery {
            transition_ts: SystemTime::now(),
            transition_time: Duration::from_secs(5),
            speed: speed,
            auto_negotiation: from.auto_negotiation,
            supported_speeds: from.supported_speeds,
            state: state,
        }
    }
}

impl FromState<Speed> for AutoNeg {
    fn from_state<'a>(from: Discovery<Speed>, port: &Port<'a>) -> Discovery<Self> {
        let state = AutoNeg {};
        // bring the port admin state down
        set_admin_state(from.state, state, port, false);

        // check if auto neg mode is supported
        // if not we can essentially transition immediately again
        let auto_neg_supported = match port.get_supported_auto_neg_mode() {
            Ok(v) => v,
            Err(e) => {
                log::error!(
                    "Port {}: state machine: {} -> {}: failed to check if auto negotiation is supported. We are assuming it is not supported: {}",
                    port,
                    from.state,
                    state,
                    e
                );
                false
            }
        };
        if !auto_neg_supported {
            log::warn!(
                "Port {}: state machine: {} -> {}: auto negotiation is not supported",
                port,
                from.state,
                state
            );
            return Discovery {
                transition_ts: SystemTime::now(),
                transition_time: Default::default(),
                speed: from.speed,
                auto_negotiation: from.auto_negotiation,
                supported_speeds: from.supported_speeds,
                state: state,
            };
        }

        // enable auto neg mode
        match port.set_auto_neg_mode(true) {
            Ok(_) => log::debug!(
                "Port {}: state machine: {} -> {}: set auto negotiation mode to on successful",
                port,
                from.state,
                state
            ),
            Err(e) => log::error!(
                "Port {}: state machine: {} -> {}: failed to set auto negotiation mode to on: {}",
                port,
                from.state,
                state,
                e
            ),
        }

        // set config mode to auto
        match port.set_auto_neg_config_mode(AutoNegConfigMode::Auto) {
            Ok(_) => log::debug!(
                "Port {}: state machine: {} -> {}: set auto negotiation config mode to auto successful",
                port,
                from.state,
                state
            ),
            Err(e) => log::error!(
                "Port {}: state machine: {} -> {}: failed to set auto negotiation config mode to auto: {}",
                port,
                from.state,
                state,
                e
            ),
        }

        // enable auto neg mode advertisement
        match port.set_advertised_auto_neg_mode(true) {
            Ok(_) => log::debug!(
                "Port {}: state machine: {} -> {}: set auto negotiation advertisement to on successful",
                port,
                from.state,
                state
            ),
            Err(e) => log::error!(
                "Port {}: state machine: {} -> {}: failed to set auto negotiation advertisement to on: {}",
                port,
                from.state,
                state,
                e
            ),
        }

        // and bring the admin state up again
        set_admin_state(from.state, state, port, true);

        // we are going to give auto negotiation up to 10 seconds to bring up a port
        Discovery {
            transition_ts: SystemTime::now(),
            transition_time: Duration::from_secs(10),
            speed: from.speed,
            auto_negotiation: true,
            supported_speeds: from.supported_speeds,
            state: state,
        }
    }
}

impl FromState<AutoNeg> for Done {
    fn from_state<'a>(from: Discovery<AutoNeg>, port: &Port<'a>) -> Discovery<Self> {
        // if we are here that means that nothing worked to bring up the port
        let state = Done { success: false };

        // bring the port admin state down
        // and we will leave it down
        set_admin_state(from.state, state, port, false);

        // as we are probably coming from auto neg being enabled, we should disable it again
        let auto_negotiation =
            disable_auto_negotiation(from.state, state, port, from.auto_negotiation);

        // we are giving up, let our users know
        log::warn!(
            "Port {}: state machine: {} -> {}: unable to bring up port",
            port,
            from.state,
            state
        );

        Discovery {
            transition_ts: SystemTime::now(),
            transition_time: Default::default(),
            speed: from.speed,
            auto_negotiation: auto_negotiation,
            supported_speeds: from.supported_speeds,
            state: state,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct Start {}
impl State for Start {}
impl Display for Start {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "START")
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct AutoNeg {}
impl State for AutoNeg {}
impl Display for AutoNeg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AUTO_NEG")
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct Speed {
    index: usize,
}
impl State for Speed {}
impl Display for Speed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SPEED[{}]", self.index)
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct Done {
    /// Describes if the discovery was successful or not.
    /// If this is false, it means that the whole discovery state machine ran, but the port did not come up.
    success: bool,
}
impl State for Done {}
impl Display for Done {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DONE[{}]", if self.success { "UP" } else { "DOWN" })
    }
}
