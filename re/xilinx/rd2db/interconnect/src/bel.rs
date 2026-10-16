use std::collections::{BTreeMap, BTreeSet};

use prjcombine_interconnect::db::{BelInfo, BelInput, PinDir, TileWireCoord};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TempBelPin {
    Input(BelInput),
    Output(BTreeSet<TileWireCoord>),
    Bidir(TileWireCoord),
}

impl TempBelPin {
    pub fn as_input(&self) -> BelInput {
        let TempBelPin::Input(wire) = *self else {
            unreachable!()
        };
        wire
    }

    pub fn as_output_mut(&mut self) -> &mut BTreeSet<TileWireCoord> {
        let TempBelPin::Output(wires) = self else {
            unreachable!()
        };
        wires
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TempBel {
    pub pins: BTreeMap<String, TempBelPin>,
}

impl TempBel {
    pub fn insert_pin(
        &mut self,
        name: impl Into<String>,
        dir: PinDir,
        wires: impl IntoIterator<Item = TileWireCoord>,
    ) {
        let name = name.into();
        let wires = BTreeSet::from_iter(wires);
        match dir {
            PinDir::Input => {
                assert_eq!(wires.len(), 1);
                let wire = wires.into_iter().next().unwrap();
                self.insert_input(name, wire);
            }
            PinDir::Output => {
                self.insert_output(name, wires);
            }
            PinDir::Inout => {
                assert_eq!(wires.len(), 1);
                let wire = wires.into_iter().next().unwrap();
                self.insert_bidir(name, wire);
            }
        }
    }

    pub fn insert_input(&mut self, name: impl Into<String>, wire: TileWireCoord) {
        assert_eq!(
            self.pins
                .insert(name.into(), TempBelPin::Input(BelInput::Fixed(wire.pos()))),
            None
        );
    }

    pub fn insert_output(
        &mut self,
        name: impl Into<String>,
        wires: impl IntoIterator<Item = TileWireCoord>,
    ) {
        assert_eq!(
            self.pins
                .insert(name.into(), TempBelPin::Output(BTreeSet::from_iter(wires))),
            None
        );
    }

    pub fn insert_bidir(&mut self, name: impl Into<String>, wire: TileWireCoord) {
        assert_eq!(self.pins.insert(name.into(), TempBelPin::Bidir(wire)), None);
    }

    pub fn remove_input(&mut self, name: &str) -> Option<BelInput> {
        match self.pins.remove(name)? {
            TempBelPin::Input(wire) => Some(wire),
            _ => {
                println!("OOPS expected {name} to be input");
                None
            }
        }
    }

    pub fn remove_output(&mut self, name: &str) -> Option<BTreeSet<TileWireCoord>> {
        match self.pins.remove(name)? {
            TempBelPin::Output(wires) => Some(wires),
            _ => {
                println!("OOPS expected {name} to be input");
                None
            }
        }
    }

    pub fn remove_bidir(&mut self, name: &str) -> Option<TileWireCoord> {
        match self.pins.remove(name)? {
            TempBelPin::Input(wire) => Some(wire.wire()),
            TempBelPin::Output(wires) => {
                assert_eq!(wires.len(), 1);
                Some(wires.into_iter().next().unwrap())
            }
            TempBelPin::Bidir(wire) => Some(wire),
        }
    }
}

#[derive(Clone, Debug)]
pub enum AnyBel {
    Temp(TempBel),
    Final(BelInfo),
}

impl From<TempBel> for AnyBel {
    fn from(value: TempBel) -> Self {
        Self::Temp(value)
    }
}

impl From<BelInfo> for AnyBel {
    fn from(value: BelInfo) -> Self {
        Self::Final(value)
    }
}
