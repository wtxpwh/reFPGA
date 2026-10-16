use std::collections::{BTreeMap, btree_map};

use prjcombine_ecp::{
    chip::Chip,
    defs::{self, bcls, bslots, wires},
};
use prjcombine_interconnect::{
    db::WireSlotIdExt,
    dir::{Dir, DirH, DirHV, DirV},
    grid::DieIdExt,
};
use prjcombine_re_lattice_naming::WireName;

use crate::{ChipContext, RcMode};

impl ChipContext<'_> {
    pub(super) fn process_pclk_scm(&mut self) -> BTreeMap<(DirHV, usize), WireName> {
        let mut hpcx = BTreeMap::new();
        let mut hpbx = BTreeMap::new();
        let mut vpsx = BTreeMap::new();
        for i in 0..12 {
            let pclk = wires::PCLK[i];
            for cell in self.edev.die_cells(Chip::DIE) {
                if !self.edev.has_bel(cell.bel(bslots::INT)) {
                    continue;
                }
                let pclk = self.naming.interconnect[&cell.wire(pclk)];
                let cur_hpbx = self.claim_single_in(pclk);
                let col_tag = self.pclk_cols[cell.col].0;
                let cell_tag = cell.with_col(col_tag);
                match hpbx.entry((cell_tag, i)) {
                    btree_map::Entry::Vacant(e) => {
                        e.insert(cur_hpbx);
                    }
                    btree_map::Entry::Occupied(e) => {
                        assert_eq!(*e.get(), cur_hpbx);
                    }
                }
            }
            for cell in self.edev.die_cells(Chip::DIE) {
                if !self.chip.columns[cell.col].pclk_drive {
                    continue;
                }
                let col_tag = self.pclk_cols[cell.col].0;
                let cell_tag = cell.with_col(col_tag);
                let Some(&cur_hpbx) = hpbx.get(&(cell_tag, i)) else {
                    continue;
                };
                let bcrd = cell.bel(bslots::INT);
                if !self.edev.has_bel(bcrd) && i == 0 {
                    self.name_bel_null(bcrd);
                }
                self.add_bel_wire(bcrd, format!("PCLK{i}"), cur_hpbx);
                let cur_vpsx = self.claim_single_in(cur_hpbx);
                self.add_bel_wire_no_claim(bcrd, format!("PCLK{i}_IN"), cur_vpsx);
                let v = if cell.row < self.chip.row_clk {
                    DirV::S
                } else {
                    DirV::N
                };
                match vpsx.entry((cell.col, v, i)) {
                    btree_map::Entry::Vacant(e) => {
                        e.insert(cur_vpsx);
                        self.claim_node(cur_vpsx);
                        let h = if cell.col < self.chip.col_clk {
                            DirH::W
                        } else {
                            DirH::E
                        };
                        let hv = DirHV { h, v };
                        let cur_hpcx = self.claim_single_in(cur_vpsx);
                        match hpcx.entry((hv, i)) {
                            btree_map::Entry::Vacant(e) => {
                                e.insert(cur_hpcx);
                            }
                            btree_map::Entry::Occupied(e) => {
                                assert_eq!(*e.get(), cur_hpcx);
                            }
                        }
                    }
                    btree_map::Entry::Occupied(e) => {
                        assert_eq!(*e.get(), cur_vpsx);
                    }
                }
            }
        }
        hpcx
    }

    pub(super) fn process_clk_edge_scm(&mut self) {
        for edge in Dir::DIRS {
            let lrbt = match edge {
                Dir::W => 'L',
                Dir::E => 'R',
                Dir::S => 'B',
                Dir::N => 'T',
            };
            let (col, row) = match edge {
                Dir::H(edge) => (self.chip.col_edge(edge), self.chip.row_clk),
                Dir::V(edge) => (self.chip.col_clk, self.chip.row_edge(edge)),
            };
            let cell_tile = Chip::DIE.cell(col, row);
            let cell = match edge {
                Dir::H(_) => cell_tile,
                Dir::V(_) => cell_tile.delta(-1, 0),
            };
            for i in 0..2 {
                let bcrd = cell_tile.bel(bslots::DCS[i]);
                let ab = ['A', 'B'][i];
                self.extract_bel_cell(bcrd, cell)
                    .name(format!("DCS{lrbt}{ab}"))
                    .input_xlat_rc_io(bcls::DCS::CLK[0], format!("JCLK0{ab}_DCS"))
                    .input_xlat_rc_io(bcls::DCS::CLK[1], format!("JCLK1{ab}_DCS"))
                    .input_xlat_rc_io(bcls::DCS::SEL[0], format!("JSEL{ab}_DCS"))
                    .output_bind_step_rc_mode(
                        bcls::DCS::DCSOUT,
                        format!("DCS{ab}_DCS"),
                        RcMode::Io,
                        wires::OUT_DCS_DCSOUT[i].cell(1),
                    )
                    .claim_pip("DCSOUT", "CLK0");
            }
            {
                let bcrd = cell_tile.bel(bslots::CLK_INT);
                let tcrd = self.edev.bel_tile(bcrd);
                let mut ex = self.extract_switchbox(bcrd);
                ex.cell(cell)
                    .input_xlat_rc_io("DCSA")
                    .input_xlat_rc_io("DCSB");
                let num_out = match edge {
                    Dir::H(_) => 24,
                    Dir::S => 8,
                    Dir::N => 4,
                };
                let prefix = match edge {
                    Dir::N => "VPFS",
                    Dir::S => "VPFN",
                    Dir::W => "HPFE",
                    Dir::E => "HPFW",
                };
                for i in 0..num_out {
                    ex.output_bind_rc_io(format!("J{prefix}{i:02}00"), wires::IMUX_PCLK[i].cell(1));
                }
                let banks: &[_] = match edge {
                    Dir::W => &[(7, 1, 0), (6, 1, 4)],
                    Dir::E => &[(2, 1, 0), (3, 1, 4)],
                    Dir::S => &[(5, 2, 0), (4, 3, 0)],
                    Dir::N => &[(1, 1, 0)],
                };
                for &(bank, ci, base) in banks {
                    if !matches!(bank, 3 | 6) {
                        for abcd in ['A', 'B', 'C', 'D'] {
                            ex.input_xlat_rc_io(format!("JCLKD{bank}{abcd}"));
                        }
                    }
                    for i in 0..4 {
                        let idx = base + i;
                        ex.input_io_rc_io(format!("JPCK{bank}{i}"), wires::IO_GCLK[idx].cell(ci));
                    }
                }
                if let Dir::H(_) = edge {
                    for i in 0..4 {
                        if ex.ctx.edev[tcrd]
                            .cells
                            .contains_id(defs::scm::tcls::CLK_W::SERDES[i])
                        {
                            for j in 0..3 {
                                ex.input_xlat_rc_io(format!("J{lrbt}SED{ii:02}01", ii = i * 3 + j));
                            }
                        }
                    }

                    for v in [DirV::S, DirV::N] {
                        let ul = match v {
                            DirV::S => 'L',
                            DirV::N => 'U',
                        };
                        for i in 0..6 {
                            let abcd = ['A', 'B', 'C', 'D', 'E', 'F'][i];
                            if i >= 4 && v == DirV::N {
                                continue;
                            }
                            ex.input_xlat_rc_io(format!("J{ul}{lrbt}CMCLK{abcd}"))
                                .input_xlat_rc_io(format!("J{ul}{lrbt}CNCLK{abcd}"));
                        }
                    }
                }
            }
        }
    }

    pub(super) fn process_clk_root_scm(&mut self, hpcx: BTreeMap<(DirHV, usize), WireName>) {
        let chip = self.chip;
        let bcrd = chip.bel_clk_root();
        let cell = bcrd.cell.delta(-1, 0);

        self.extract_bel_cell(bcrd.bel(bslots::CLKTEST), cell)
            .name("TESTCK")
            .suffix("TESTCK");

        let mut ex = self.extract_switchbox(bcrd);
        ex.cell(cell)
            .input_xlat_rc("JCIBLLQ")
            .input_xlat_rc("JCIBULQ")
            .input_xlat_rc("JCIBURQ");
        for edge in Dir::DIRS {
            let hv = match edge {
                Dir::H(_) => 'H',
                Dir::V(_) => 'V',
            };
            let num_inps = match edge {
                Dir::H(_) => 24,
                Dir::S => 8,
                Dir::N => 4,
            };
            for i in 0..num_inps {
                ex.input_xlat_rc(format!("J{hv}PF{ndir}{i:02}01", ndir = !edge));
            }
        }
        for hv in DirHV::DIRS {
            let cidx = match hv {
                DirHV::SW => 0,
                DirHV::SE => 1,
                DirHV::NW => 2,
                DirHV::NE => 3,
            };
            for i in 0..12 {
                ex.output_bind(hpcx[&(hv, i)], wires::PCLK[i].cell(cidx));
            }
        }
    }
}
