use prjcombine_ecp::{
    chip::{ChipKind, NexusKind, SpecialLocKey},
    defs::{bslots, tcls},
};
use prjcombine_interconnect::dir::Dir;

use crate::ChipContext;

impl ChipContext<'_> {
    pub(super) fn process_lram(&mut self) {
        if !matches!(self.chip.kind, ChipKind::Nexus(_)) {
            return;
        }
        for (tcid, bels) in [
            (tcls::LRAM_W_JE5D15, [(0, 23), (0, 14), (0, 5)].as_slice()),
            (tcls::LRAM_E_JE5D15, [(0, 9), (0, 8)].as_slice()),
            (tcls::LRAM_E_JE5D30_A, [(0, 5)].as_slice()),
            (tcls::LRAM_E_JE5D30_B, [(0, 1)].as_slice()),
            (tcls::LRAM_E_JD5D_A, [(0, 0)].as_slice()),
            (tcls::LRAM_E_JD5D_B, [(0, 0)].as_slice()),
            (tcls::LRAM_E_JD5D_C, [(0, 0)].as_slice()),
            (tcls::LRAM_E_JD5U, [(0, 0)].as_slice()),
            (tcls::LRAM_N_JD5D, [(0, 0)].as_slice()),
        ] {
            for &tcrd in &self.edev.tile_index[tcid] {
                for (i, &(dx, dy)) in bels.iter().enumerate() {
                    let mut cell = tcrd.delta(dx, dy);
                    if matches!(self.chip.kind, ChipKind::Nexus(NexusKind::LargePcs(_)))
                        && tcrd.cell == self.chip.special_loc[&SpecialLocKey::Lram(Dir::N, 0)]
                    {
                        cell = cell.step(Dir::E, 1);
                    }
                    self.extract_bel_cell(tcrd.bel(bslots::LRAM[i]), cell)
                        .name_rc("LRAM_CORE")
                        .suffix("LRAM_CORE");
                }
            }
        }
        for (tcid, dy) in [(tcls::ALU_JE5D30, 6), (tcls::ALU_JD5D, 0)] {
            for &tcrd in &self.edev.tile_index[tcid] {
                self.extract_bel_cell(tcrd.bel(bslots::ALU), tcrd.delta(0, dy))
                    .name_rc("ALUREG_CORE")
                    .suffix("ALUREG_CORE");
            }
        }
    }
}
