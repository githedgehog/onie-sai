use crate::oniesai::port::PhysicalPort;
use sai::port::BreakoutModeType;
use std::fmt::Display;
use thiserror::Error;

pub(crate) trait State
where
    Self: std::fmt::Debug + Display + Clone + Copy,
{
}

pub(crate) trait FromState<S: State>: Sized {
    fn from_state(from: Discovery<S>, port: &mut PhysicalPort<'_, '_>) -> Discovery<Self>
    where
        Self: State;
}

pub(crate) trait IntoState<S: State>: Sized {
    fn into_state(self, port: &mut PhysicalPort<'_, '_>) -> Discovery<S>
    where
        S: State;
}

impl<S1: State, S2: State> IntoState<S2> for Discovery<S1>
where
    S2: FromState<S1>,
{
    fn into_state(self, port: &mut PhysicalPort<'_, '_>) -> Discovery<S2> {
        S2::from_state(self, port)
    }
}

#[derive(Debug, Clone)]
pub(crate) enum DiscoveryStateMachine {
    BreakoutMode(Discovery<BreakoutMode>),
    Done(Discovery<Done>),
}

impl DiscoveryStateMachine {
    /// initializes a new logical port discovery state machine
    pub(crate) fn new(
        idx: usize,
        auto_discovery_with_breakout: bool,
        current_breakout_mode: BreakoutModeType,
        supported_breakout_modes: Vec<BreakoutModeType>,
    ) -> Self {
        DiscoveryStateMachine::BreakoutMode(Discovery::new(
            idx,
            auto_discovery_with_breakout,
            current_breakout_mode,
            supported_breakout_modes,
        ))
    }

    pub(crate) fn is_done(&self) -> bool {
        match self {
            DiscoveryStateMachine::Done(_) => true,
            _ => false,
        }
    }

    pub(crate) fn is_done_and_success(&self) -> bool {
        match self {
            DiscoveryStateMachine::Done(v) => v.state.success,
            _ => false,
        }
    }

    pub(crate) fn can_step(&self, port: &PhysicalPort<'_, '_>) -> bool {
        match self {
            DiscoveryStateMachine::BreakoutMode(s) => s.is_done(port),
            DiscoveryStateMachine::Done(_) => false,
        }
    }

    pub(crate) fn step(self, port: &mut PhysicalPort<'_, '_>) -> Self {
        match self {
            DiscoveryStateMachine::Done(_) => self,
            // if we cannot step yet (logical port state machines aren't done yet, we just return ourselves)
            DiscoveryStateMachine::BreakoutMode(ref discovery) if !discovery.is_done(port) => self,
            // if we can step to done success, we do so
            DiscoveryStateMachine::BreakoutMode(discovery) if discovery.is_done_success(port) => {
                DiscoveryStateMachine::Done(discovery.into_state(port))
            }
            // if we could step but we have discovery with breakouts disabled, then we are done as well
            DiscoveryStateMachine::BreakoutMode(discovery)
                if discovery.is_done(port) && !discovery.auto_discovery_with_breakout =>
            {
                DiscoveryStateMachine::Done(discovery.into_state(port))
            }
            // nothing more to try, we are done
            DiscoveryStateMachine::BreakoutMode(discovery)
                if discovery.left_breakout_modes.len() == 0 =>
            {
                DiscoveryStateMachine::Done(discovery.into_state(port))
            }
            DiscoveryStateMachine::BreakoutMode(discovery) => {
                DiscoveryStateMachine::BreakoutMode(discovery.into_state(port))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Discovery<S: State> {
    auto_discovery_with_breakout: bool,
    current_breakout_mode: BreakoutModeType,
    left_breakout_modes: Vec<BreakoutModeType>,
    state: S,
}

impl<S: State> Discovery<S> {
    /// Checks if we can advance our state machine. We can only continue if all state machines of all logical ports are done
    fn is_done(&self, port: &PhysicalPort<'_, '_>) -> bool {
        for lp in port.ports.iter() {
            match &lp.sm {
                None => return false,
                Some(sm) => {
                    if !sm.is_done() {
                        return false;
                    }
                }
            }
        }
        return true;
    }

    /// Checks if we can move to the successful done state. All state machines for all logical ports must be in the done state.
    /// If just one of them is successful, we consider the physical port state machine successful.
    fn is_done_success(&self, port: &PhysicalPort<'_, '_>) -> bool {
        if !self.is_done(port) {
            return false;
        }
        for lp in port.ports.iter() {
            match &lp.sm {
                None => return false,
                Some(sm) => {
                    if sm.is_done_and_success() {
                        return true;
                    }
                }
            }
        }
        return false;
    }
}

impl Discovery<BreakoutMode> {
    /// initializes a new logical port discovery state machine
    pub(crate) fn new(
        idx: usize,
        auto_discovery_with_breakout: bool,
        current_breakout_mode: BreakoutModeType,
        supported_breakout_modes: Vec<BreakoutModeType>,
    ) -> Self {
        let ret = Self::internal_new(
            auto_discovery_with_breakout,
            current_breakout_mode,
            supported_breakout_modes,
        );
        log::debug!(
            "Physical Port {}: state machine: {}: initialized. Left breakout modes: {:?}",
            idx,
            ret.state,
            ret.left_breakout_modes.clone()
        );
        ret
    }

    fn internal_new(
        auto_discovery_with_breakout: bool,
        current_breakout_mode: BreakoutModeType,
        supported_breakout_modes: Vec<BreakoutModeType>,
    ) -> Self {
        let left_breakout_modes: Vec<BreakoutModeType> = supported_breakout_modes
            .into_iter()
            .filter(|&x| x != current_breakout_mode)
            .collect();
        Discovery {
            auto_discovery_with_breakout: auto_discovery_with_breakout,
            current_breakout_mode: current_breakout_mode,
            left_breakout_modes: left_breakout_modes,
            state: BreakoutMode {
                mode: current_breakout_mode,
            },
        }
    }
}

impl FromState<BreakoutMode> for BreakoutMode {
    fn from_state(from: Discovery<BreakoutMode>, port: &mut PhysicalPort<'_, '_>) -> Discovery<Self>
    where
        Self: State,
    {
        // get the next breakout mode that we want to try
        // if there are no more breakout modes left, then this is a bug (and we cannot model this better here except maybe for a panic)
        let next_breakout_mode = match from.left_breakout_modes.first() {
            Some(v) => *v,
            None => {
                log::error!(
                    "Physical Port {}: state machine: {}: no breakout modes left to try. State machine transition error!",
                    port.idx,
                    from.state,
                );
                return from;
            }
        };
        let new_state = BreakoutMode {
            mode: next_breakout_mode,
        };
        log::debug!(
            "Physical Port {}: state machine: {} -> {}: starting transition",
            port.idx,
            from.state,
            new_state
        );

        // calculate all the ports that we need to create for the next breakout mode by using the lanes of the port
        let new_ports = match calculate_new_ports(&next_breakout_mode, &port.lanes) {
            Ok(v) => v,
            Err(e) => {
                log::error!(
                    "Physical Port {}: state machine: {} -> {}: error calculating new ports: {}",
                    port.idx,
                    from.state,
                    new_state,
                    e
                );
                return from;
            }
        };
        log::debug!(
            "Physical Port {}: state machine: {} -> {}: calculated new ports: {:?}",
            port.idx,
            from.state,
            new_state,
            new_ports.clone()
        );

        // remove all old ports now
        port.remove_ports();

        // and create new ports from calculation
        // this function will add the newly created ports to the physical port
        for new_port_hw_lanes in new_ports.into_iter() {
            port.create_port(next_breakout_mode, new_port_hw_lanes);
        }

        // as we know a transceiver is present for this port
        // we go ahead and immediately create the hif and rif for all those ports
        port.create_hifs_and_rifs();

        // now we initialize new state machines for the logical ports
        for lp in port.ports.iter_mut() {
            lp.sm = Some(super::logicalport::DiscoveryStateMachine::new(
                &lp.port,
                lp.supported_speeds.clone(),
                lp.speed,
                lp.auto_negotiation,
            ));
        }

        // return with the new state
        // we can just rely on the new function because it will filter out the current breakout mode
        let ret = Discovery::internal_new(
            from.auto_discovery_with_breakout,
            next_breakout_mode,
            from.left_breakout_modes.clone(),
        );
        log::debug!(
            "Physical Port {}: state machine: {} -> {}: finished transition (Left breakout modes: {:?})",
            port.idx,
            from.state,
            ret.state,
            ret.left_breakout_modes.clone(),
        );
        ret
    }
}

#[derive(Error, Debug)]
enum CalcError {
    #[error("invalid number of hw lanes: {0}")]
    InvalidNumberOfHwLanes(usize),

    #[error("invalid number of hw lanes for breakout mode {1}: {0}")]
    InvalidNumberOfHwLanesForMode(usize, BreakoutModeType),

    #[error("unknown breakout mode: {0}")]
    UnknownBreakoutMode(i32),
}

fn calculate_new_ports(
    mode: &BreakoutModeType,
    lanes: &Vec<u32>,
) -> Result<Vec<Vec<u32>>, CalcError> {
    // validate number of lanes first
    let lanes_len = lanes.len();
    if lanes_len == 0 || lanes_len > 4 {
        return Err(CalcError::InvalidNumberOfHwLanes(lanes.len()));
    }

    let mut ret: Vec<Vec<u32>> = Vec::new();
    match mode {
        BreakoutModeType::Unknown(v) => return Err(CalcError::UnknownBreakoutMode(*v)),
        BreakoutModeType::FourLanes => {
            if lanes_len % 4 != 0 {
                return Err(CalcError::InvalidNumberOfHwLanesForMode(lanes_len, *mode));
            }
            ret.push(lanes.clone());
        }
        BreakoutModeType::TwoLanes => {
            if lanes_len % 2 != 0 {
                return Err(CalcError::InvalidNumberOfHwLanesForMode(lanes_len, *mode));
            }
            for lane_pair in lanes.chunks(2) {
                ret.push(lane_pair.to_vec());
            }
        }
        BreakoutModeType::OneLane => {
            for lane in lanes.iter() {
                ret.push(vec![*lane]);
            }
        }
    }
    Ok(ret)
}

impl FromState<BreakoutMode> for Done {
    fn from_state(from: Discovery<BreakoutMode>, port: &mut PhysicalPort<'_, '_>) -> Discovery<Self>
    where
        Self: State,
    {
        let success = from.is_done_success(port);
        let ret = Discovery {
            auto_discovery_with_breakout: from.auto_discovery_with_breakout,
            current_breakout_mode: from.current_breakout_mode,
            left_breakout_modes: from.left_breakout_modes,
            state: Done { success: success },
        };
        if success {
            // we can consider this port operational now
            // if it wasn't before, we log this as well
            if !port.oper_status {
                port.oper_status = true;
                log::info!(
                    "Physical Port {}: state machine: {} -> {}: successfully brought up port",
                    port.idx,
                    from.state,
                    ret.state
                );
            }
        } else {
            // if we failed to bring up the port, we consider it non operational
            port.oper_status = false;
            log::warn!(
                "Physical Port {}: state machine: {} -> {}: failed to bring up port",
                port.idx,
                from.state,
                ret.state
            );
        }
        ret
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct BreakoutMode {
    mode: BreakoutModeType,
}
impl State for BreakoutMode {}
impl Display for BreakoutMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BREAKOUT_MODE[{:?}]", self.mode)
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Done {
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
