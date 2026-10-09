use prjcombine_ecp::{chip::Chip, defs::wires};
use prjcombine_interconnect::{db::WireSlotIdExt, dir::DirHV, grid::DieIdExt};

use crate::ChipContext;

impl ChipContext<'_> {
    pub(super) fn process_clk_machxo(&mut self) {
        let chip = self.chip;
        let bcrd = chip.bel_clk_root();
        let cell = if chip.rows.len() == 21 {
            Chip::DIE.cell(chip.col_clk - 1, chip.row_clk - 1)
        } else {
            Chip::DIE.cell(chip.col_clk - 1, chip.row_clk)
        };

        let mut ex = self.extract_switchbox(bcrd);
        ex.cell(cell)
            .input_io_rc("JGCLK0", wires::IO_GCLK_N[0].cell(3))
            .input_io_rc("JGCLK1", wires::IO_GCLK_N[1].cell(3))
            .input_io_rc("JGCLK2", wires::IO_GCLK_S[0].cell(3))
            .input_io_rc("JGCLK3", wires::IO_GCLK_S[1].cell(3));

        for i in 0..3 {
            for j in 0..4 {
                ex.input_xlat_rc(format!("JCIBCTL{i}{j}"));
                if j == 2 && chip.try_bel_pll(DirHV::NW, 0).is_some() {
                    continue;
                }
                if j == 3 && chip.try_bel_pll(DirHV::SW, 0).is_some() {
                    continue;
                }
                ex.input_xlat_rc(format!("JCIBCLK{i}{j}"));
            }
        }

        for (quad, clkop, clkos, clkok) in [
            (DirHV::SW, "JLLMMCLKA", "JLLMNCLKA", "JLFPSC1"),
            (DirHV::NW, "JULMMCLKA", "JULMNCLKA", "JLFPSC0"),
        ] {
            if chip.try_bel_pll(quad, 0).is_some() {
                ex.input_xlat_rc(clkop)
                    .input_xlat_rc(clkos)
                    .input_xlat_rc(clkok);
            }
        }

        for i in 0..8 {
            ex.output_int(wires::PCLK[i].cell(3));
        }
    }
}
