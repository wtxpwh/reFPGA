use std::collections::BTreeMap;

use prjcombine_ecp::{
    chip::{Chip, ChipKind, RowKind, SpecialLocKey},
    defs::{self, bcls, bslots, tcls, wires},
};
use prjcombine_entity::EntityId;
use prjcombine_interconnect::{
    coord::{DieIdExt, WireCoord},
    db::{CellSlotId, CellSlotIdExt, WireSlotIdExt},
    dir::{Dir, DirH, DirHV, DirV},
};
use prjcombine_re_lattice_naming::WireName;

use crate::ChipContext;

impl ChipContext<'_> {
    pub(super) fn process_pclk_ecp2(&mut self) {
        for &tcrd in &self.edev.tile_index[tcls::PCLK_DRIVER] {
            let bcrd = tcrd.bel(bslots::PCLK_DRIVER);
            let mut ex = self.extract_switchbox(bcrd);
            for i in 0..8 {
                ex.progbuf_output_int(wires::PCLK[i].cell(0))
                    .input_int(wires::PCLK_ROOT[i].cell(0));
            }
        }
    }

    pub(super) fn process_sclk_root_ecp2(&mut self) -> BTreeMap<WireCoord, WireName> {
        let mut sclk_root_out = BTreeMap::new();
        let mut prev_row = BTreeMap::new();
        let mut prev = self.chip.row_s();
        for (row, rd) in &self.chip.rows {
            if matches!(rd.kind, RowKind::Io | RowKind::Ebr | RowKind::Dsp) {
                prev_row.insert(row, prev);
                prev = row;
                let bcrd = Chip::DIE
                    .cell(self.chip.col_clk, row)
                    .bel(bslots::SCLK_H_SPLITTER);
                let tcrd = self.edev.bel_tile(bcrd);
                let mut ex = self.extract_switchbox(bcrd);
                for i in 0..8 {
                    ex.progbuf_output_int(wires::SCLK_H[i].cell(0))
                        .progbuf_output_int(wires::SCLK_H[i].cell(1));
                    for (h, cslot) in [
                        (DirH::W, defs::ecp2::tcls::SCLK_H_ROOT::W),
                        (DirH::E, defs::ecp2::tcls::SCLK_H_ROOT::E),
                    ] {
                        let cell = ex.ctx.edev[tcrd].cells[cslot];
                        let wname = ex.ctx.naming.interconnect[&cell.wire(wires::SCLK_H[i])];
                        let wname_out = ex.ctx.pips_bwd[&wname]
                            .iter()
                            .copied()
                            .find(|wn| ex.ctx.naming.strings[wn.suffix].starts_with("VSC"))
                            .unwrap();
                        ex.input_bind(wname_out, cslot.wire(wires::SCLK_ROOT[i]));
                        sclk_root_out.insert((row, h, i), wname_out);
                    }
                }
            }
        }

        let mut sclk_root = BTreeMap::new();
        for h in [DirH::W, DirH::E] {
            let col = self.chip.col_edge(h);
            for i in 0..8 {
                let wire = sclk_root_out[&(self.chip.row_clk, h, i)];
                sclk_root.insert(
                    Chip::DIE
                        .cell(col, self.chip.row_n())
                        .wire(wires::SCLK_ROOT[i]),
                    wire,
                );

                let wire = sclk_root_out[&(prev_row[&self.chip.row_clk], h, i)];
                let wire = self.find_single_in(wire);
                self.claim_node(wire);
                sclk_root.insert(
                    Chip::DIE
                        .cell(col, self.chip.row_s())
                        .wire(wires::SCLK_ROOT[i]),
                    wire,
                );
            }
        }

        for (row, rd) in &self.chip.rows {
            if matches!(rd.kind, RowKind::Io | RowKind::Ebr | RowKind::Dsp)
                && row != self.chip.row_s()
            {
                for h in [DirH::W, DirH::E] {
                    for i in 0..8 {
                        let wire_n = sclk_root_out[&(row, h, i)];
                        let wire_s = sclk_root_out[&(prev_row[&row], h, i)];
                        if row < self.chip.row_clk {
                            self.claim_pip(wire_s, wire_n);
                        } else if row == self.chip.row_clk {
                            let wire_n = sclk_root[&Chip::DIE
                                .cell(self.chip.col_edge(h), self.chip.row_s())
                                .wire(wires::SCLK_ROOT[i])];
                            self.claim_pip(wire_s, wire_n);
                        } else {
                            self.claim_pip(wire_n, wire_s);
                        }
                    }
                }
            }
        }

        sclk_root
    }

    pub(super) fn process_clk_ecp2(&mut self, sclk_root: BTreeMap<WireCoord, WireName>) {
        let chip = self.chip;
        let bcrd = chip.bel_clk_root();
        let cell = bcrd.step(Dir::W, 1);

        let mut ex = self.extract_switchbox(bcrd);
        ex.cell(cell)
            .input_io_rc("JLPIO0", wires::OUT_IO_INCK[0].cell(30))
            .input_io_rc("JLPIO1", wires::OUT_IO_INCK[0].cell(31))
            .input_io_rc("JRPIO0", wires::OUT_IO_INCK[0].cell(32))
            .input_io_rc("JRPIO1", wires::OUT_IO_INCK[0].cell(33))
            .input_io_rc("JBPIO0", wires::OUT_IO_INCK[0].cell(34))
            .input_io_rc("JBPIO1", wires::OUT_IO_INCK[0].cell(35))
            .input_io_rc("JTPIO0", wires::OUT_IO_INCK[0].cell(36))
            .input_io_rc("JTPIO1", wires::OUT_IO_INCK[0].cell(37))
            .input_xlat_rc("JLCDIV1")
            .input_xlat_rc("JLCDIV2")
            .input_xlat_rc("JLCDIV4")
            .input_xlat_rc("JLCDIV8")
            .input_xlat_rc("JRCDIV1")
            .input_xlat_rc("JRCDIV2")
            .input_xlat_rc("JRCDIV4")
            .input_xlat_rc("JRCDIV8");
        for (wire, name) in sclk_root {
            ex.ctx.bind_wire(name, wire);
            ex.output_xlat_no_claim(name);
        }

        for ci in 0..4 {
            for i in 0..6 {
                ex.output_int(wires::PCLK_ROOT[i].cell(ci));
            }
        }

        for (ci, prefix) in ["LL", "LR", "UL", "UR"].into_iter().enumerate() {
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
                    let name = match (dir, idx) {
                        (Dir::W, 0) => "JCIBLLQ0",
                        (Dir::W, 1) => "JCIBULQ0",
                        (Dir::E, 0) => "JCIBLRQ2",
                        (Dir::E, 1) => "JCIBLRQ0",
                        (Dir::E, 2) => "JCIBURQ0",
                        (Dir::E, 3) => "JCIBURQ2",
                        (Dir::S, 0) => "JCIBLLQ1",
                        (Dir::S, 1) => "JCIBLRQ1",
                        (Dir::N, 0) => "JCIBULQ1",
                        (Dir::N, 1) => "JCIBURQ1",
                        _ => unreachable!(),
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

        if chip.kind == ChipKind::Xp2 {
            for hv in DirHV::DIRS {
                let ci_pll = match hv {
                    DirHV::SW => 38,
                    DirHV::SE => 39,
                    DirHV::NW => 40,
                    DirHV::NE => 41,
                };
                if chip.try_bel_pll(hv, 0).is_some() {
                    let (wn_clkop_pll, wn_clkos_pll, wn_clkok_pll, wn_clkok2_pll) = match hv {
                        DirHV::SW => ("JLLCMCLKA", "JLLCNCLKA", "JLFPSC3", "JLFPSC4"),
                        DirHV::SE => ("JLRCMCLKA", "JLRCNCLKA", "JRFPSC3", "JRFPSC4"),
                        DirHV::NW => ("JULCMCLKA", "JULCNCLKA", "JLFPSC1", "JLFPSC2"),
                        DirHV::NE => ("JURCMCLKA", "JURCNCLKA", "JRFPSC1", "JRFPSC2"),
                    };
                    ex.input_xlat_rc_nudge(wn_clkop_pll, CellSlotId::from_idx(ci_pll));
                    ex.input_xlat_rc_nudge(wn_clkos_pll, CellSlotId::from_idx(ci_pll));
                    ex.input_xlat_rc_nudge(wn_clkok_pll, CellSlotId::from_idx(ci_pll));
                    ex.input_xlat_rc_nudge(wn_clkok2_pll, CellSlotId::from_idx(ci_pll));
                }
            }
        } else {
            for (loc, _bel_pll) in chip.plls() {
                if loc.quad.v == DirV::S && loc.idx == 0 {
                    let (wn_clkop_pll, wn_clkos_pll, wn_clkok_pll, wn_clkop_dll, wn_clkos_dll) =
                        match (chip.kind, loc.quad.h) {
                            (ChipKind::Ecp2, DirH::W) => (
                                "JLLMMCLKA",
                                "JLLMNCLKA",
                                "JLFPSC2",
                                "JLLMMCLKB",
                                "JLLMNCLKB",
                            ),
                            (ChipKind::Ecp2, DirH::E) => (
                                "JLRMMCLKA",
                                "JLRMNCLKA",
                                "JRFPSC2",
                                "JLRMMCLKB",
                                "JLRMNCLKB",
                            ),
                            (ChipKind::Ecp2M, DirH::W) => (
                                "JLLMMCLKC",
                                "JLLMNCLKC",
                                "JLFPSC3",
                                "JLLMMCLKB",
                                "JLLMNCLKB",
                            ),
                            (ChipKind::Ecp2M, DirH::E) => (
                                "JLRMMCLKC",
                                "JLRMNCLKC",
                                "JRFPSC3",
                                "JLRMMCLKB",
                                "JLRMNCLKB",
                            ),
                            _ => unreachable!(),
                        };
                    ex.input_xlat_rc(wn_clkop_pll);
                    ex.input_xlat_rc(wn_clkos_pll);
                    ex.input_xlat_rc(wn_clkok_pll);
                    ex.input_xlat_rc(wn_clkop_dll);
                    ex.input_xlat_rc(wn_clkos_dll);
                } else {
                    let (wn_clkop_pll, wn_clkos_pll, wn_clkok_pll) = match (loc.quad, loc.idx) {
                        (DirHV::SW, 1) => ("JLLMMCLKA", "JLLMNCLKA", "JLFPSC2"),
                        (DirHV::SE, 1) => ("JLRMMCLKA", "JLRMNCLKA", "JRFPSC2"),
                        (DirHV::NW, 0) => ("JULMMCLKA", "JULMNCLKA", "JLFPSC0"),
                        (DirHV::NE, 0) => ("JURMMCLKA", "JURMNCLKA", "JRFPSC0"),
                        (DirHV::NW, 1) => ("JULMMCLKB", "JULMNCLKB", "JLFPSC1"),
                        (DirHV::NE, 1) => ("JURMMCLKB", "JURMNCLKB", "JRFPSC1"),
                        _ => unreachable!(),
                    };
                    ex.input_xlat_rc(wn_clkop_pll);
                    ex.input_xlat_rc(wn_clkos_pll);
                    ex.input_xlat_rc(wn_clkok_pll);
                }
            }
        }

        if chip.kind == ChipKind::Ecp2M {
            for hv in DirHV::DIRS {
                if chip.try_bel_serdes_ecp2(hv).is_some() {
                    let corner = match hv {
                        DirHV::SW => "LL",
                        DirHV::SE => "LR",
                        DirHV::NW => "UL",
                        DirHV::NE => "UR",
                    };
                    ex.input_xlat_rc(format!("J{corner}SQ_TX_F_CLK"));
                    ex.input_xlat_rc(format!("J{corner}SQ_TX_H_CLK"));
                }
            }
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
                    .input_xlat_rc(bcls::DCS::SEL[0], format!("{prefix}SEL{i}_DCS"))
                    .output_xlat_rc(bcls::DCS::DCSOUT, format!("{prefix}DCSOUT{i}_DCS"))
                    .claim_pip("DCSOUT", "CLK0")
                    .claim_pip("DCSOUT", "CLK1");
            }
        }
    }
}
