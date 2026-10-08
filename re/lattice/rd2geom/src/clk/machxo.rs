use std::collections::BTreeMap;

use prjcombine_ecp::{
    chip::{Chip, SpecialIoKey},
    defs::wires,
};
use prjcombine_interconnect::{
    db::WireSlotIdExt,
    dir::{Dir, DirHV},
    grid::DieIdExt,
};

use crate::ChipContext;

impl ChipContext<'_> {
    pub(super) fn process_clk_machxo(&mut self) {
        let bcrd = self.chip.bel_clk_root();
        let cell = if self.chip.rows.len() == 21 {
            Chip::DIE.cell(self.chip.col_clk - 1, self.chip.row_clk - 1)
        } else {
            Chip::DIE.cell(self.chip.col_clk - 1, self.chip.row_clk)
        };

        let mut inputs = BTreeMap::new();

        for i in 0..3 {
            for j in 0..4 {
                let wname = self.rc_wire(cell, &format!("JCIBCTL{i}{j}"));
                let wire = self.xlat_input(bcrd, wname).wire();
                inputs.insert(wname, wire);
                if j == 2 && self.chip.try_bel_pll(DirHV::NW, 0).is_some() {
                    continue;
                }
                if j == 3 && self.chip.try_bel_pll(DirHV::SW, 0).is_some() {
                    continue;
                }
                let wname = self.rc_wire(cell, &format!("JCIBCLK{i}{j}"));
                let wire = self.xlat_input(bcrd, wname).wire();
                inputs.insert(wname, wire);
            }
        }

        for (wire, key, gwire) in [
            (
                "JGCLK0",
                SpecialIoKey::Clock(Dir::N, 0),
                wires::IO_GCLK_N[0],
            ),
            (
                "JGCLK1",
                SpecialIoKey::Clock(Dir::N, 1),
                wires::IO_GCLK_N[1],
            ),
            (
                "JGCLK2",
                SpecialIoKey::Clock(Dir::S, 0),
                wires::IO_GCLK_S[0],
            ),
            (
                "JGCLK3",
                SpecialIoKey::Clock(Dir::S, 1),
                wires::IO_GCLK_S[1],
            ),
        ] {
            let gwire = gwire.cell(3);
            let wire = self.rc_wire(cell, wire);
            let wire_io = self.get_special_io_wire_in(key);
            self.claim_pip(wire, wire_io);
            inputs.insert(wire, gwire);
        }

        for (quad, cidx, clkop, clkos, clkok) in [
            (DirHV::SW, 6, "JLLMMCLKA", "JLLMNCLKA", "JLFPSC1"),
            (DirHV::NW, 7, "JULMMCLKA", "JULMNCLKA", "JLFPSC0"),
        ] {
            let Some(bel_pll) = self.chip.try_bel_pll(quad, 0) else {
                continue;
            };
            let pll_cell = bel_pll.cell;
            let clkop = self.rc_wire(cell, clkop);
            let clkos = self.rc_wire(cell, clkos);
            let clkok = self.rc_wire(cell, clkok);
            let clkop_pll = self.rc_wire(pll_cell, "JCLKOP_PLL");
            let clkos_pll = self.rc_wire(pll_cell, "JCLKOS_PLL");
            let clkok_pll = self.rc_wire(pll_cell, "JCLKOK_PLL");
            self.claim_pip(clkop, clkop_pll);
            self.claim_pip(clkos, clkos_pll);
            self.claim_pip(clkok, clkok_pll);
            inputs.insert(clkop, wires::OUT_PLL_CLKOP.cell(cidx));
            inputs.insert(clkos, wires::OUT_PLL_CLKOS.cell(cidx));
            inputs.insert(clkok, wires::OUT_PLL_CLKOK.cell(cidx));
        }

        let mut extractor = self.extract_switchbox(bcrd);
        for (name, wire) in inputs {
            extractor.input(name, wire);
        }

        for i in 0..8 {
            extractor.output_int(wires::PCLK[i].cell(3));
        }
    }
}
