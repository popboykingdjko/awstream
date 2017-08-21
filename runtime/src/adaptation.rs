//! Adapatation algorithm implementation (described as in Figure 6).

/// Signal
#[derive(Debug, Clone, Copy)]
pub enum Signal {
    MonitorTimer,
    QueueCongest(f64),
    ProbeDone,
    ConfigMax,
    QueueEmpty,
    QueueEmptyAtMaxConfig,
}

#[derive(Debug, Clone, Copy)]
pub enum Action {
    NoOp,
    AdvanceConfig,
    AdjustConfig,
    StartProbe,
    IncreaseProbePace,
    StopProbe,
}

#[derive(Debug, Clone, Copy)]
/// States of the rate adaptation algorithm.
enum State {
    Startup,
    Degrade,
    Steady,
    Probe,
}

pub struct Adaptation {
    state: State,
}

impl Adaptation {
    pub fn new() -> Adaptation {
        Adaptation { state: State::Startup }
    }

    pub fn transit(&mut self, signal: Signal) -> Action {
        info!("state: {:?}, signal: {:?}", self.state, signal);
        let action = match (self.state, signal) {
            (State::Startup, Signal::QueueEmpty) => {
                // transition 1
                Action::AdvanceConfig
            }
            (State::Startup, Signal::QueueEmptyAtMaxConfig) => {
                // transition 2, queue is empty and config at max
                self.state = State::Steady;
                Action::NoOp
            }
            (State::Startup, Signal::QueueCongest(_latency)) => {
                // transition 3
                self.state = State::Degrade;
                Action::AdjustConfig
            }
            (State::Degrade, Signal::QueueCongest(_latency)) => {
                // transition 4
                self.state = State::Degrade;
                Action::AdjustConfig
            }
            (State::Degrade, Signal::QueueEmpty) |
            (State::Degrade, Signal::QueueEmptyAtMaxConfig) => {
                // transition 5
                self.state = State::Steady;
                Action::NoOp
            }
            (State::Steady, Signal::QueueCongest(_latency)) => {
                // transition 6
                self.state = State::Degrade;
                Action::AdjustConfig
            }
            (State::Steady, Signal::QueueEmpty) => {
                // transition 7
                self.state = State::Probe;
                Action::StartProbe
            }
            (State::Probe, Signal::QueueCongest(_latency)) => {
                // transtion 8
                self.state = State::Steady;
                Action::StopProbe
            }
            (State::Probe, Signal::ProbeDone) => {
                // transition 9
                self.state = State::Steady;
                Action::AdvanceConfig
            }
            (State::Probe, Signal::QueueEmpty) => {
                // transition 10
                Action::IncreaseProbePace
            }
            _ => {
                error!("Unhandled state {:?} and signal {:?}", self.state, signal);
                unimplemented!{}
            }
        };
        info!("state: {:?}, action: {:?}", self.state, action);
        action
    }
}