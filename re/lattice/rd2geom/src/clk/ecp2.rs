use std::collections::BTreeMap;

use prjcombine_ecp::{
    chip::{Chip, ChipKind, RowKind, SpecialLocKey},
    defs::{bcls, bslots, wires},
};
use prjcombine_entity::EntityId;
use prjcombine_interconnect::{
    db::{CellSlotId, WireSlotIdExt},
    dir::{Dir, DirH, DirHV, DirV},
    grid::{DieIdExt, WireCoord},
};
use prjcombine_re_lattice_naming::WireName;

use crate::ChipContext;

impl ChipContext<'_> {
    pub(super) fn process_sclk_root_ecp2(&mut self) -> BTreeMap<WireCoord, WireName> {
        let mut sdclk_root_out = BTreeMap::new();
        let mut prev_row = BTreeMap::new();
        let mut prev = self.chip.row_s();
        for (row, rd) in &self.chip.rows {
            if matches!(rd.kind, RowKind::Io | RowKind::Ebr | RowKind::Dsp) {
                prev_row.insert(row, prev);
                prev = row;
                let bcrd = Chip::DIE
                    .cell(self.chip.col_clk, row)
                    .bel(bslots::HSDCLK_SPLITTER);
                for (h, ci_base) in [(DirH::W, 0), (DirH::E, 4)] {
                    for i in 0..8 {
                        let wire = wires::RSCLK_H[i / 4 * 4].cell(ci_base + i % 4);
                        let wire = self.edev.bel_wire(bcrd, wire);
                        let wname = self.naming.interconnect[&wire];
                        let wname_out = self.pips_bwd[&wname]
                            .iter()
                            .copied()
                            .find(|wn| self.naming.strings[wn.suffix].starts_with("VSC"))
                            .unwrap();
                        sdclk_root_out.insert((row, h, i), wname_out);
                        self.add_sb_wire(bcrd, wires::SCLK_ROOT[i].cell(ci_base), wname_out);
                        self.claim_pip(wname, wname_out);
                    }
                }
            }
        }

        let mut sclk_root = BTreeMap::new();
        for h in [DirH::W, DirH::E] {
            let col = self.chip.col_edge(h);
            for i in 0..8 {
                let wire = sdclk_root_out[&(self.chip.row_clk, h, i)];
                sclk_root.insert(
                    Chip::DIE
                        .cell(col, self.chip.row_n())
                        .wire(wires::SCLK_ROOT[i]),
                    wire,
                );

                let wire = sdclk_root_out[&(prev_row[&self.chip.row_clk], h, i)];
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
                        let wire_n = sdclk_root_out[&(row, h, i)];
                        let wire_s = sdclk_root_out[&(prev_row[&row], h, i)];
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
        let cell = Chip::DIE.cell(self.chip.col_clk - 1, self.chip.row_clk);

        let mut ex = self.extract_switchbox(bcrd);
        ex.cell(cell)
            .input_io_rc("JLPIO0", wires::OUT_IO_PADDI[0].cell(30))
            .input_io_rc("JLPIO1", wires::OUT_IO_PADDI[0].cell(31))
            .input_io_rc("JRPIO0", wires::OUT_IO_PADDI[0].cell(32))
            .input_io_rc("JRPIO1", wires::OUT_IO_PADDI[0].cell(33))
            .input_io_rc("JBPIO0", wires::OUT_IO_PADDI[0].cell(34))
            .input_io_rc("JBPIO1", wires::OUT_IO_PADDI[0].cell(35))
            .input_io_rc("JTPIO0", wires::OUT_IO_PADDI[0].cell(36))
            .input_io_rc("JTPIO1", wires::OUT_IO_PADDI[0].cell(37))
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

        let mut pclk_outs = BTreeMap::new();
        for (cell_idx, hv) in [DirHV::SW, DirHV::SE, DirHV::NW, DirHV::NE]
            .into_iter()
            .enumerate()
        {
            for i in 0..8 {
                let tw = wires::PCLK[i].cell(cell_idx);
                let wire = ex.ctx.edev.bel_wire(bcrd, tw);
                let wname = ex.ctx.naming.interconnect[&wire];
                let wname_out = ex.ctx.find_single_in(wname);
                ex.ctx.bind_wire(wname_out, wire);
                if i < 6 {
                    ex.output_xlat(wname_out);
                } else {
                    ex.ctx.add_sb_wire(ex.bcrd, tw, wname_out);
                }
                pclk_outs.insert((hv, i), wname_out);
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
                } else {
                    for ci in 0..4 {
                        for i in 0..6 {
                            for w in [
                                wires::OUT_PLL_CLKOP[0],
                                wires::OUT_PLL_CLKOS[0],
                                wires::OUT_PLL_CLKOK,
                                wires::OUT_PLL_CLKOK2,
                            ] {
                                ex.edge(wires::PCLK[i].cell(ci), w.cell(ci_pll));
                            }
                        }
                        for i in 0..2 {
                            for w in [
                                wires::OUT_PLL_CLKOS[0],
                                wires::OUT_PLL_CLKOK,
                                wires::OUT_PLL_CLKOK2,
                            ] {
                                ex.edge(wires::IMUX_DCS_CLKA[i].cell(ci), w.cell(ci_pll));
                                ex.edge(wires::IMUX_DCS_CLKB[i].cell(ci), w.cell(ci_pll));
                            }
                            if hv.h == DirH::W {
                                ex.edge(
                                    wires::IMUX_DCS_CLKB[i].cell(ci),
                                    wires::OUT_PLL_CLKOP[0].cell(ci_pll),
                                );
                            } else {
                                ex.edge(
                                    wires::IMUX_DCS_CLKA[i].cell(ci),
                                    wires::OUT_PLL_CLKOP[0].cell(ci_pll),
                                );
                            }
                        }
                    }
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
            if chip.kind == ChipKind::Ecp2 {
                for (ci_base, hv) in [(40, DirHV::NW), (42, DirHV::NE)] {
                    for idx in 0..2 {
                        let ci_pll = ci_base + idx;
                        if chip.try_bel_pll(hv, idx).is_none() {
                            for ci in 0..4 {
                                for i in 0..6 {
                                    for w in [
                                        wires::OUT_PLL_CLKOP[0],
                                        wires::OUT_PLL_CLKOS[0],
                                        wires::OUT_PLL_CLKOK,
                                    ] {
                                        ex.edge(wires::PCLK[i].cell(ci), w.cell(ci_pll));
                                    }
                                }
                                for i in 0..2 {
                                    for w in [wires::OUT_PLL_CLKOP[0], wires::OUT_PLL_CLKOS[0]] {
                                        ex.edge(wires::IMUX_DCS_CLKA[i].cell(ci), w.cell(ci_pll));
                                        ex.edge(wires::IMUX_DCS_CLKB[i].cell(ci), w.cell(ci_pll));
                                    }
                                    if hv.h == DirH::W {
                                        ex.edge(
                                            wires::IMUX_DCS_CLKA[i].cell(ci),
                                            wires::OUT_PLL_CLKOK.cell(ci_pll),
                                        );
                                    } else {
                                        ex.edge(
                                            wires::IMUX_DCS_CLKB[i].cell(ci),
                                            wires::OUT_PLL_CLKOK.cell(ci_pll),
                                        );
                                    }
                                }
                            }
                        }
                    }
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
                } else {
                    let ci_serdes = match hv {
                        DirHV::SW => 46,
                        DirHV::SE => 47,
                        DirHV::NW => 48,
                        DirHV::NE => 49,
                    };
                    for ci in 0..4 {
                        for i in 0..6 {
                            ex.edge(
                                wires::PCLK[i].cell(ci),
                                [wires::OUT_SERDES_FF_TX_H_CLK, wires::OUT_SERDES_FF_TX_F_CLK]
                                    [i % 2]
                                    .cell(ci_serdes),
                            );
                        }
                        for i in 0..2 {
                            ex.edge(
                                wires::IMUX_DCS_CLKA[i].cell(ci),
                                wires::OUT_SERDES_FF_TX_H_CLK.cell(ci_serdes),
                            );
                            ex.edge(
                                wires::IMUX_DCS_CLKB[i].cell(ci),
                                wires::OUT_SERDES_FF_TX_F_CLK.cell(ci_serdes),
                            );
                        }
                    }
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
                    .input_xlat_rc(bcls::DCS::SEL, format!("{prefix}SEL{i}_DCS"))
                    .output_xlat_rc(bcls::DCS::DCSOUT, format!("{prefix}DCSOUT{i}_DCS"))
                    .claim_pip("DCSOUT", "CLK0")
                    .claim_pip("DCSOUT", "CLK1");
            }
        }

        for (col, cd) in &self.chip.columns {
            if !cd.pclk_break && col != self.chip.col_w() {
                continue;
            }
            let h = if col < self.chip.col_clk {
                DirH::W
            } else {
                DirH::E
            };
            'cells: for mut cell in self.edev.column(cell.die, col) {
                let v = if cell.row < self.chip.row_clk {
                    DirV::S
                } else {
                    DirV::N
                };
                let hv = DirHV { h, v };
                while !self.edev.has_bel(cell.bel(bslots::INT)) {
                    if cell.col == self.chip.col_e() {
                        continue 'cells;
                    }
                    cell.col += 1;
                    if self.chip.columns[cell.col].pclk_break {
                        continue 'cells;
                    }
                }
                for i in 0..8 {
                    let pclk = cell.wire(wires::PCLK[i]);
                    self.claim_pip_int_out(pclk, pclk_outs[&(hv, i)]);
                }
            }
        }
    }
}
