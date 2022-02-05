use crate::Protocol;
use crate::protocol::utils::InfraConstRange;
use crate::receiver::{DecoderState, DecoderStateMachine, Status};

pub struct Mitsubishi {

}

#[derive(Debug, Copy, Clone)]
pub struct MCmd {
    raw: [u8; 18],
}

impl Protocol for Mitsubishi { type Cmd = MCmd; }

pub struct State {
    cmd: MCmd,
    bitindex: usize,
    status: MStatus,
}

impl DecoderState for State {
    fn reset(&mut self) {
        self.cmd.raw.iter_mut().for_each(|m| *m = 0);
        self.bitindex = 0;
        self.status = MStatus::Init;
    }
}

impl State {
    fn add_val(&mut self, v: bool) {

        let byte_index = self.bitindex / 8;
        let bit_in_byte = self.bitindex % 8;


        self.cmd.raw[byte_index] |= (v as u8) << (7 - bit_in_byte);

        self.bitindex += 1;
    }
}

#[derive(Copy, Clone, Debug)]
pub enum MStatus {
    Init,
    InitLow,
    Data(bool),
    Done,
}

#[derive(Debug)]
enum MPulse {
    LeaderHigh,
    LeaderLow,
    Data,
    DataLow,
    DataHigh,
    NotAPulseWidth,
}


impl From<usize> for MPulse {
    fn from(v: usize) -> Self {
        match v {
            0 => MPulse::LeaderHigh,
            1 => MPulse::LeaderLow,
            2 => MPulse::Data,
            3 => MPulse::DataLow,
            4 => MPulse::DataHigh,
            _ => MPulse::NotAPulseWidth,
        }
    }
}

impl Into<Status> for MStatus {
    fn into(self) -> Status {
        match self {
            MStatus::Init => Status::Idle,
            MStatus::InitLow => Status::Receiving,
            MStatus::Data(_) => Status::Receiving,
            MStatus::Done => Status::Done,
        }
    }
}

impl DecoderStateMachine for Mitsubishi {
    type State = State;
    type RangeData = InfraConstRange<5>;
    type InternalStatus = MStatus;

    fn state() -> Self::State {
        State {
            cmd: MCmd {
                raw: [0; 18],
            },
            bitindex: 0,
            status: MStatus::Init
        }
    }

    fn ranges(resolution: u32) -> Self::RangeData {
        InfraConstRange::new(&[
            (3450, 10),
            (1600, 10),
            (400, 20),
            (1200, 10),
            (400, 20)
        ], resolution)
    }

    fn event_full(res: &mut Self::State, rd: &Self::RangeData, edge: bool, dt: u32) -> Self::InternalStatus {
        use MPulse::*;

        let pulsewidth = rd.find::<MPulse>(dt).unwrap_or(MPulse::NotAPulseWidth);

        let bitindex = res.bitindex;
        //println!("{:?}, {dt} {:?} {bitindex}", res.status, pulsewidth);

        res.status = match (res.status, pulsewidth) {
            (MStatus::Init, LeaderHigh) => MStatus::InitLow,
            (MStatus::InitLow, LeaderLow) => MStatus::Data(false),
            (MStatus::Data(false), Data) => MStatus::Data(true),
            (MStatus::Data(true), DataHigh | Data) => {
                res.add_val(false);

                if res.bitindex == 18 * 8 {
                    MStatus::Done
                } else {
                    MStatus::Data(false)
                }

            }
            (MStatus::Data(true), DataLow ) => {
                res.add_val(true);
                if res.bitindex == 18 * 8 {
                    MStatus::Done
                } else {
                    MStatus::Data(false)
                }
            },

            (_, _) => MStatus::Init,
        };

        res.status
    }

    fn command(state: &Self::State) -> Option<Self::Cmd> {
        Some(state.cmd)
    }
}