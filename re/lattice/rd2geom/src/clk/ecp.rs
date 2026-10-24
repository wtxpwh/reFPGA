use prjcombine_ecp::{
    chip::SpecialLocKey,
    defs::{bcls, bslots, wires},
};
use prjcombine_interconnect::{
    db::WireSlotIdExt,
    dir::{Dir, DirHV},
};

use crate::ChipContext;

impl ChipContext<'_> {
    pub(super) fn process_clk_ecp(&mut self) {
        let chip = self.chip;
        let bcrd = chip.bel_clk_root();
        let cell = bcrd.step(Dir::W, 1);

        let mut ex = self.extract_switchbox(bcrd);

        ex.cell(cell)
            .input_io_rc("JLPIO", wires::OUT_IO_INCK[0].cell(24))
            .input_io_rc("JRPIO", wires::OUT_IO_INCK[0].cell(25))
            .input_io_rc("JBPIO", wires::OUT_IO_INCK[0].cell(26))
            .input_io_rc("JTPIO", wires::OUT_IO_INCK[0].cell(27));

        for (ci, prefix) in ["LL", "LR", "UL", "UR"].into_iter().enumerate() {
            for i in [0, 1, 4, 5, 6, 7] {
                ex.output_int(wires::PCLK[i].cell(ci));
            }
            for i in 0..2 {
                let clka = ex.ctx.rc_wire(cell, &format!("{prefix}CLK{i}A_DCS"));
                let clka_in = ex.ctx.find_single_in(clka);
                let clkb = ex.ctx.rc_wire(cell, &format!("{prefix}CLK{i}B_DCS"));
                let clkb_in = ex.ctx.find_single_in(clkb);
                ex.output_bind(clka_in, wires::IMUX_DCS_CLKA[i].cell(ci))
                    .output_bind(clkb_in, wires::IMUX_DCS_CLKB[i].cell(ci));
            }
        }

        for &loc in chip.special_loc.keys() {
            match loc {
                SpecialLocKey::PclkIn(dir, idx) => {
                    assert_eq!(idx, 0);
                    let name = match dir {
                        Dir::W => "JCIBLLQ",
                        Dir::E => "JCIBURQ",
                        Dir::S => "JCIBLRQ",
                        Dir::N => "JCIBULQ",
                    };
                    ex.input_xlat_rc(name);
                }
                SpecialLocKey::SclkIn(dir, idx) => {
                    let name = match dir {
                        Dir::W => format!("JCIBL{idx}"),
                        Dir::E => format!("JCIBR{idx}"),
                        Dir::S => format!("JCIBB{idx}"),
                        Dir::N => format!("JCIBT{idx}"),
                    };
                    ex.input_xlat_rc(name);
                }
                _ => (),
            }
        }

        for (loc, _bcrd) in chip.plls() {
            let (wn_clkop, wn_clkos, wn_clkok) = match loc.quad {
                DirHV::SW => ("JLLMMCLKA", "JLLMNCLKA", "JLFPSC1"),
                DirHV::SE => ("JLRMMCLKA", "JLRMNCLKA", "JRFPSC1"),
                DirHV::NW => ("JULMMCLKA", "JULMNCLKA", "JLFPSC0"),
                DirHV::NE => ("JURMMCLKA", "JURMNCLKA", "JRFPSC0"),
            };
            ex.input_xlat_rc(wn_clkop)
                .input_xlat_rc(wn_clkos)
                .input_xlat_rc(wn_clkok);
        }

        core::mem::drop(ex);

        for (prefix, slots) in [
            ("LL", bslots::DCS_SW),
            ("LR", bslots::DCS_SE),
            ("UL", bslots::DCS_NW),
            ("UR", bslots::DCS_NE),
        ] {
            for i in 0..2 {
                self.extract_bel_cell(bcrd.bel(slots[i]), cell)
                    .name(format!("{prefix}DCS{i}"))
                    .input_xlat_rc(bcls::DCS::CLK[0], format!("{prefix}CLK{i}A_DCS"))
                    .input_xlat_rc(bcls::DCS::CLK[1], format!("{prefix}CLK{i}B_DCS"))
                    .input_xlat_rc(bcls::DCS::SEL[0], format!("J{prefix}SEL{i}_DCS"))
                    .output_xlat_rc(bcls::DCS::DCSOUT, format!("{prefix}DCSOUT{i}_DCS"));
            }
        }
    }
}
