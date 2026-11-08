use std::collections::BTreeMap;

use bincode::{Decode, Encode};
use itertools::Itertools;
use prjcombine_interconnect::{coord::BelPadCoord, db::IntDb};

use crate::chip::ChipKind;

#[derive(Clone, Debug, Eq, PartialEq, Hash, Encode, Decode)]
pub struct Bond {
    pub kind: ChipKind,
    pub pins: BTreeMap<String, Option<BelPadCoord>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExpandedBond<'a> {
    pub bond: &'a Bond,
    pub ios: BTreeMap<BelPadCoord, String>,
}

impl Bond {
    pub fn expand(&self) -> ExpandedBond<'_> {
        let mut ios = BTreeMap::new();
        let power = match self.kind {
            ChipKind::Xc2000 => crate::xc2000::bslots::POWER,
            ChipKind::Xc3000 | ChipKind::Xc3000A => crate::xc3000::bslots::POWER,
            ChipKind::Xc4000
            | ChipKind::Xc4000A
            | ChipKind::Xc4000H
            | ChipKind::Xc4000E
            | ChipKind::Xc4000Ex
            | ChipKind::Xc4000Xla
            | ChipKind::Xc4000Xv
            | ChipKind::SpartanXl => crate::xc4000::bslots::POWER,
            ChipKind::Xc5200 => crate::xc5200::bslots::POWER,
        };
        for (name, &pad) in &self.pins {
            if let Some(pad) = pad {
                if pad.slot == power {
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
        writeln!(o, "\tkind {};", self.kind)?;
        for (pin, pad) in self.pins.iter().sorted_by_key(|(k, _)| pad_sort_key(k)) {
            if let Some(pad) = pad {
                writeln!(o, "\tpin {pin} = {pad};", pad = pad.to_string(db))?;
            } else {
                writeln!(o, "\tpin {pin} = nc;")?;
            }
        }
        Ok(())
    }
}
