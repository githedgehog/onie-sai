#![allow(dead_code, unused_variables)]
use sai::port::Port;
use std::time::{Duration, SystemTime};

pub(super) trait State {}

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

/*
pub trait From<T>: Sized {
    /// Converts to this type from the input type.
    #[rustc_diagnostic_item = "from_fn"]
    #[must_use]
    #[stable(feature = "rust1", since = "1.0.0")]
    fn from(value: T) -> Self;
}

pub trait Into<T>: Sized {
    /// Converts this type into the (usually inferred) input type.
    #[must_use]
    #[stable(feature = "rust1", since = "1.0.0")]
    fn into(self) -> T;
}

impl<T, U> Into<U> for T
where
    U: From<T>,
{
    /// Calls `U::from(self)`.
    ///
    /// That is, this conversion is whatever the implementation of
    /// <code>[From]&lt;T&gt; for U</code> chooses to do.
    #[inline]
    fn into(self) -> U {
        U::from(self)
    }
}
*/

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
    pub(super) fn new(supported_speeds: Vec<u32>) -> Self {
        DiscoveryStateMachine::Start(Discovery::new(supported_speeds))
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
                    DiscoveryStateMachine::Done(Discovery::success(v))
                }
                DiscoveryStateMachine::AutoNeg(v) => {
                    DiscoveryStateMachine::Done(Discovery::success(v))
                }
                DiscoveryStateMachine::Speed(v) => {
                    DiscoveryStateMachine::Done(Discovery::success(v))
                }
                DiscoveryStateMachine::Done(v) => {
                    DiscoveryStateMachine::Done(Discovery::success(v))
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
                        DiscoveryStateMachine::AutoNeg(v.into_state(port))
                    }
                    DiscoveryStateMachine::AutoNeg(v) => {
                        DiscoveryStateMachine::Speed(v.into_state(port))
                    }
                    DiscoveryStateMachine::Speed(v) if v.state.index == 0 => {
                        DiscoveryStateMachine::Done(v.into_state(port))
                    }
                    DiscoveryStateMachine::Speed(v) => self,
                    DiscoveryStateMachine::Done(v) => self,
                }
            }
        }
    }
}

#[derive(Debug)]
pub(super) struct Discovery<S: State> {
    transition_ts: SystemTime,
    transition_time: Duration,
    auto_negotiation: bool,
    supported_speeds: Vec<u32>,
    state: S,
}

impl<S: State> Discovery<S> {
    fn can_step(&self) -> bool {
        self.transition_ts.elapsed().unwrap_or_default() > self.transition_time
    }
}

impl Discovery<Start> {
    fn new(supported_speeds: Vec<u32>) -> Self {
        let mut s = supported_speeds;
        s.sort();
        Discovery {
            transition_ts: SystemTime::now(),
            transition_time: Duration::from_secs(5),
            auto_negotiation: false,
            supported_speeds: s,
            state: Start {},
        }
    }
}

impl Discovery<Done> {
    fn success<T: State>(from: Discovery<T>) -> Self {
        Discovery {
            transition_ts: SystemTime::now(),
            transition_time: Default::default(),
            auto_negotiation: from.auto_negotiation,
            supported_speeds: from.supported_speeds,
            state: Done { success: true },
        }
    }
}

impl FromState<Start> for AutoNeg {
    fn from_state<'a>(from: Discovery<Start>, _port: &Port<'a>) -> Discovery<Self> {
        todo!("implement");
    }
}

impl FromState<AutoNeg> for Speed {
    fn from_state<'a>(from: Discovery<AutoNeg>, _port: &Port<'a>) -> Discovery<Self> {
        todo!("implement");
    }
}

#[derive(Debug)]
pub(super) struct Start {}
impl State for Start {}

#[derive(Debug)]
pub(super) struct AutoNeg {
    enable: bool,
}
impl State for AutoNeg {}

#[derive(Debug)]
pub(super) struct Speed {
    index: usize,
}
impl State for Speed {}

#[derive(Debug)]
pub(super) struct Done {
    success: bool,
}
impl State for Done {}
