use prjcombine_ecp::chip::ChipKind;

use super::ChipContext;

mod ecp;
mod ecp2;
mod machxo2;
mod nexus;

impl ChipContext<'_> {
    pub fn process_plc(&mut self) {
        match self.chip.kind {
            ChipKind::Scm | ChipKind::Ecp | ChipKind::Xp | ChipKind::MachXo => {
                self.process_plc_ecp()
            }
            ChipKind::Ecp2 | ChipKind::Ecp2M | ChipKind::Xp2 | ChipKind::Ecp3(_) => {
                self.process_plc_ecp2()
            }
            ChipKind::MachXo2(_) | ChipKind::Ecp4 | ChipKind::Ecp5 | ChipKind::Crosslink => {
                self.process_plc_machxo2()
            }
            ChipKind::Nexus(_) => self.process_plc_nexus(),
        }
    }
}
