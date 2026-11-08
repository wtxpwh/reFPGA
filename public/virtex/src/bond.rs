use bincode::{Decode, Encode};
use itertools::Itertools;
use prjcombine_interconnect::{coord::BelPadCoord, db::IntDb};
use std::collections::{BTreeMap, BTreeSet};

use crate::defs::{bcls, bslots};

#[derive(Clone, Debug, Eq, PartialEq, Hash, Encode, Decode)]
pub struct Bond {
    pub pins: BTreeMap<String, Vec<BelPadCoord>>,
    pub vref: BTreeSet<BelPadCoord>,
    pub diffp: BTreeSet<BelPadCoord>,
    pub diffn: BTreeSet<BelPadCoord>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExpandedBond<'a> {
    pub bond: &'a Bond,
    pub ios: BTreeMap<BelPadCoord, String>,
}

impl Bond {
    pub fn expand(&self) -> ExpandedBond<'_> {
        let mut ios = BTreeMap::new();
        for (name, pads) in &self.pins {
            for &pad in pads {
                if pad.slot == bslots::BANK_WE || pad.slot == bslots::BANK_SN {
                    continue;
                }
                if pad.slot == bslots::GLOBAL
                    && matches!(
                        pad.pad,
                        bcls::GLOBAL::GND | bcls::GLOBAL::VCCINT | bcls::GLOBAL::VCCAUX
                    )
                {
                    continue;
                }
                ios.insert(pad, name.clone());
            }
        }
        ExpandedBond { bond: self, ios }
    }
}

fn pad_sort_key(name: &str) -> (usize, &str, u32) {
    let pos = name.find(|x: char| x.is_ascii_digit()).unwrap();
    (pos, &name[..pos], name[pos..].parse().unwrap())
}

impl Bond {
    pub fn dump(&self, o: &mut dyn std::io::Write, db: &IntDb) -> std::io::Result<()> {
        for (pin, pads) in self.pins.iter().sorted_by_key(|(k, _)| pad_sort_key(k)) {
            if pads.is_empty() {
                writeln!(o, "\tpin {pin} = nc;")?;
            } else {
                writeln!(
                    o,
                    "\tpin {pin} = {pads};",
                    pads = pads.iter().map(|x| x.to_string(db)).join(" + ")
                )?;
            }
        }
        for pad in &self.vref {
            writeln!(o, "\tvref {pad};", pad = pad.to_string(db))?;
        }
        for pad in &self.diffp {
            writeln!(o, "\tdiffp {pad};", pad = pad.to_string(db))?;
        }
        for pad in &self.diffn {
            writeln!(o, "\tdiffn {pad};", pad = pad.to_string(db))?;
        }
        Ok(())
    }
}
