use prjcombine_ecp::{
    chip::{Chip, RowKind, SpecialIoKey, SpecialLocKey},
    defs::{bcls, bslots, tcls, wires},
};
use prjcombine_entity::EntityId;
use prjcombine_interconnect::{
    coord::{BelCoord, CellCoord, DieIdExt},
    db::{CellSlotId, CellSlotIdExt, WireSlotIdExt},
    dir::{Dir, DirHV},
};

use crate::{ChipContext, RcMode};

impl ChipContext<'_> {
    pub(super) fn prep_io_ecp4(&mut self) {
        for cell in self.edev.die_cells(Chip::DIE) {
            for iob in 0..4 {
                let bcrd = cell.bel(bslots::IO[iob]);
                if self.edev.has_bel(bcrd) {
                    let (cell_io, abcd) = self.xlat_io_loc_ecp4(bcrd);
                    let paddi_pio = self.rc_io_wire(cell_io, &format!("JPADDI{abcd}_PIO"));
                    self.bind_wire(paddi_pio, bcrd.wire(wires::OUT_IO_INCK[iob]));
                }
            }
        }
    }

    pub(super) fn process_bc_ecp4(&mut self) {
        let has_bank0 = self.chip.special_loc.contains_key(&SpecialLocKey::Bc(0));
        for (&key, &cell_loc) in &self.chip.special_loc {
            let SpecialLocKey::Bc(bank) = key else {
                continue;
            };
            let (col, row, suffix, is_corner) = match bank {
                0 => (self.chip.col_w(), self.chip.row_n(), "T", true),
                1 if !has_bank0 => (self.chip.col_w(), self.chip.row_n(), "T", true),
                1 if has_bank0 => (self.chip.col_clk - 1, self.chip.row_n(), "TL", false),
                2 if has_bank0 => (self.chip.col_clk - 1, self.chip.row_n(), "TR", false),
                2 if !has_bank0 => (self.chip.col_e(), self.chip.row_n(), "T", true),
                3 => (self.chip.col_e(), self.chip.row_n(), "T", true),
                4 => (self.chip.col_e(), self.chip.row_n(), "R", true),
                5 => (self.chip.col_e(), self.chip.row_s(), "R", false),
                6 => (self.chip.col_w(), self.chip.row_s(), "L", false),
                7 => (self.chip.col_w(), self.chip.row_n(), "L", true),
                _ => unreachable!(),
            };
            let cell = Chip::DIE.cell(col, row);
            for (bel, name, pin_out, pin_in, pname_in) in [
                (bslots::BCPG, "BCPG", "PGENO", bcls::BCPG::PGENI, "PGENI"),
                (
                    bslots::BCINRD,
                    "BCINRD",
                    "INRDENO",
                    bcls::BCINRD::INRDENI,
                    "INRDENI",
                ),
                (
                    bslots::BCLVDSO,
                    "BCLVDSO",
                    "LVDSENO",
                    bcls::BCLVDSO::LVDSENI,
                    "LVDSENI",
                ),
                (
                    bslots::BCPUSL,
                    "BCPUSL",
                    "PUSLENO",
                    bcls::BCPUSL::PUSLENI,
                    "PUSLENI",
                ),
            ] {
                let bcrd = cell_loc.bel(bel);

                let wire_in = if is_corner {
                    self.rc_corner_wire(cell, &format!("J{pname_in}_{name}_{suffix}"))
                } else {
                    self.rc_io_wire(cell, &format!("J{pname_in}_{name}_{suffix}"))
                };
                let wire_out = if is_corner {
                    self.rc_corner_wire(cell, &format!("{pin_out}_{name}_{suffix}"))
                } else {
                    self.rc_io_wire(cell, &format!("{pin_out}_{name}_{suffix}"))
                };
                self.claim_pip(wire_out, wire_in);
                let wire_out_out = self.claim_single_out(wire_out);

                self.extract_bel_cell(bcrd, cell)
                    .name(format!("{name}{bank}"))
                    .input_xlat(pin_in, wire_in)
                    .add_wire(pin_out, wire_out)
                    .add_wire(format!("{pin_out}_OUT"), wire_out_out);
            }
            {
                let bcrd = cell_loc.bel(bslots::BREFTEST);

                let mut bel = self.extract_bel_cell(bcrd, cell);
                bel.name(format!("BREFTEST{bank}"));

                for i in 0..6 {
                    let wire_in = if is_corner {
                        bel.ctx
                            .rc_corner_wire(cell, &format!("JTESTIN{i}_BREFTEST_{suffix}"))
                    } else {
                        bel.ctx
                            .rc_io_wire(cell, &format!("JTESTIN{i}_BREFTEST_{suffix}"))
                    };
                    let wire_out = if is_corner {
                        bel.ctx
                            .rc_corner_wire(cell, &format!("JTESTOUT{i}_BREFTEST_{suffix}"))
                    } else {
                        bel.ctx
                            .rc_io_wire(cell, &format!("JTESTOUT{i}_BREFTEST_{suffix}"))
                    };
                    bel.input_xlat(bcls::BREFTEST::TESTIN[i], wire_in)
                        .output_xlat(bcls::BREFTEST::TESTOUT[i], wire_out);
                }
            }
        }
        {
            let cell = self.chip.special_loc[&SpecialLocKey::Bc(if has_bank0 { 3 } else { 2 })];
            self.extract_bel(cell.bel(bslots::PVTTEST))
                .name("PVTTEST")
                .suffix("PVTTEST");
            self.extract_bel(cell.bel(bslots::PVTCAL))
                .name("PVTCAL")
                .suffix("PVTCAL");
        }
    }

    pub(super) fn process_eclk_ecp4(&mut self) {
        for (edge, num_banks) in [(Dir::N, 4), (Dir::W, 2), (Dir::E, 2)] {
            let cell_tile = self.chip.clk_emid(edge);
            let bcrd_int = cell_tile.bel(bslots::CLK_INT);
            let ci_base = if self.edev[self.edev.bel_tile(bcrd_int)].class == tcls::CLK_N_L {
                9
            } else {
                5
            };

            for bank_idx in 0..num_banks {
                let (bank, cell) = match edge {
                    Dir::N => (bank_idx, cell_tile.delta(-2 + (bank_idx as i32), 0)),
                    Dir::W => (5 - bank_idx, cell_tile.delta(0, -1 + (bank_idx as i32))),
                    Dir::E => (6 + bank_idx, cell_tile.delta(0, -1 + (bank_idx as i32))),
                    Dir::S => unreachable!(),
                };
                if !self
                    .chip
                    .special_loc
                    .contains_key(&SpecialLocKey::Bc(bank as u32))
                {
                    continue;
                }
                for eclk_idx in 0..4 {
                    let wname = self.rc_io_wire(cell, &format!("JECLKO_ECLKSYNC{eclk_idx}"));
                    self.claim_node(wname);
                    self.bind_wire(
                        wname,
                        cell_tile.wire(wires::OUT_ECLKSYNC_ECLKO[bank_idx * 4 + eclk_idx]),
                    );
                }
            }
            for bank_idx in 0..num_banks {
                let (bank, cell) = match edge {
                    Dir::N => (bank_idx, cell_tile.delta(-2 + (bank_idx as i32), 0)),
                    Dir::W => (5 - bank_idx, cell_tile.delta(0, -1 + (bank_idx as i32))),
                    Dir::E => (6 + bank_idx, cell_tile.delta(0, -1 + (bank_idx as i32))),
                    Dir::S => unreachable!(),
                };
                if !self
                    .chip
                    .special_loc
                    .contains_key(&SpecialLocKey::Bc(bank as u32))
                {
                    continue;
                }
                {
                    let mut ex = self.extract_switchbox(bcrd_int);
                    ex.cell(cell);
                    for eclk_idx in 0..4 {
                        let ci_eclk = match bank {
                            0 => 18,
                            3 => 19,
                            1..3 => ci_base + bank - 2,
                            _ => ci_base + bank_idx - 1,
                        };
                        let eclki = ex
                            .ctx
                            .rc_io_wire(cell, &format!("ECLKI_ECLKSYNC{eclk_idx}"));
                        let eclki_in = ex.ctx.find_single_in(eclki);
                        ex.output_bind(
                            eclki_in,
                            wires::IMUX_ECLKSYNC_ECLKI[bank_idx * 4 + eclk_idx].cell(ci_base),
                        )
                        .input_xlat_rc_io(format!("JECLKCIB{eclk_idx}"))
                        .input_xlat_rc_io(format!("JBRGECLK{eclk_idx}"))
                        .input_xlat_rc_io(format!("SYNCECLK{eclk_idx}"));

                        let wire = wires::ECLK[eclk_idx].cell(ci_eclk);
                        let wname = ex.ctx.rc_io_wire(cell, &format!("JECLK{eclk_idx}"));
                        let outs = ex.ctx.xlat_output(ex.bcrd, wname);
                        assert_eq!(outs.len(), 1);
                        let out = outs.into_iter().next().unwrap();
                        ex.permabuf(out, wire).output_bind(wname, wire);
                        if edge == Dir::N {
                            if bank_idx != 0
                                && ex
                                    .ctx
                                    .edev
                                    .has_bel(bcrd_int.bel(bslots::ECLKSYNC[(bank_idx - 1) * 4]))
                            {
                                ex.input_xlat_rc_io(format!("JPREVECLK{eclk_idx}"));
                            }
                            if bank_idx != 3
                                && ex
                                    .ctx
                                    .edev
                                    .has_bel(bcrd_int.bel(bslots::ECLKSYNC[(bank_idx + 1) * 4]))
                            {
                                ex.input_xlat_rc_io(format!("JNEXTECLK{eclk_idx}"));
                            }
                        } else {
                            ex.input_xlat_rc_io(format!("JNEIGHBORECLK{eclk_idx}"));
                        }
                    }
                    for i in 0..8 {
                        if !ex.ctx.edev.has_bel(bcrd_int.bel(bslots::DLLDEL[i])) {
                            continue;
                        }
                        let name = match edge {
                            Dir::N => ["00", "01", "10", "11", "20", "21", "30", "31"][i],
                            Dir::W => ["60", "61", "70", "71"][i],
                            Dir::E => ["50", "51", "40", "41"][i],
                            Dir::S => unreachable!(),
                        };
                        ex.input_xlat_rc_io(format!("JPCLKT{name}"));
                    }
                    let plls = match edge {
                        Dir::W => [DirHV::SW, DirHV::NW],
                        Dir::E => [DirHV::SE, DirHV::NE],
                        Dir::N => [DirHV::NW, DirHV::NE],
                        Dir::S => unreachable!(),
                    };
                    for hv in plls {
                        let corner = match hv {
                            DirHV::SW => "LL",
                            DirHV::SE => "LR",
                            DirHV::NW => "UL",
                            DirHV::NE => "UR",
                        };
                        for i in 0..2 {
                            for pin in ["CLKOP", "CLKOS"] {
                                ex.input_xlat_rc_io(format!("J{corner}CPLL{i}{pin}"));
                            }
                        }
                    }
                }
                for eclk_idx in 0..4 {
                    let bcrd = cell_tile.bel(bslots::ECLKSYNC[bank_idx * 4 + eclk_idx]);
                    if !self.edev.has_bel(bcrd) {
                        continue;
                    }

                    let eclk_mux = self.rc_io_wire(cell, &format!("JECLK{eclk_idx}"));
                    let eclk = self.pips_fwd[&eclk_mux]
                        .iter()
                        .copied()
                        .find(|wn| self.naming.strings[wn.suffix].starts_with("BANK"))
                        .unwrap();
                    self.claim_pip(eclk, eclk_mux);
                    self.bind_wire(eclk, self.io_int_wires[&eclk_mux][0]);

                    self.extract_bel_cell(bcrd, cell)
                        .name(format!("ECLKSYNC{eclk_idx}_BK{bank}"))
                        .input_xlat_rc_io(bcls::ECLKSYNC::STOP, format!("JSTOP_ECLKSYNC{eclk_idx}"))
                        .input_xlat_rc_io(
                            bcls::ECLKSYNC::ECLKI,
                            format!("ECLKI_ECLKSYNC{eclk_idx}"),
                        )
                        .output_xlat_rc_io(
                            bcls::ECLKSYNC::ECLKO,
                            format!("JECLKO_ECLKSYNC{eclk_idx}"),
                        )
                        .add_wire("ECLK", eclk)
                        .claim_pip("ECLKO", "ECLKI");
                }
            }
        }
        let cell_n = self.chip.clk_emid(Dir::N);
        let cell_w = self.chip.clk_emid(Dir::W);
        let cell_e = self.chip.clk_emid(Dir::E);
        for (lrt, cell_tile, cell, n) in [
            ('T', cell_n, cell_n.delta(-1, 0), 7),
            ('L', cell_w, cell_w, 12),
            ('R', cell_e, cell_e, 12),
        ] {
            let bcrd = cell_tile.bel(bslots::CLKTEST_ECLK);
            let mut bel = self.extract_bel_cell(bcrd, cell);
            bel.name(format!("CLKTEST_{lrt}ECLK"));

            for i in 0..24 {
                if i < n {
                    bel.input_xlat_rc_io(bcls::CLKTEST::TESTIN[i], format!("JTESTIN{i}_CLKTEST"));
                } else {
                    bel.add_wire_rc_io(format!("TESTIN{i}"), format!("JTESTIN{i}_CLKTEST"));
                }
            }
        }

        for (cell, base) in [(cell_w, 2), (cell_e, 0)] {
            for i in 0..2 {
                let eclk_idx = base + i;
                let bcrd_cs = cell.bel(bslots::ECLKBRIDGECS[i]);

                self.extract_bel(bcrd_cs)
                    .name(format!("ECLKBRIDGECS{eclk_idx}"))
                    .input_xlat_rc_io(
                        bcls::ECLKBRIDGECS::SEL,
                        format!("JSEL_ECLKBRIDGECS{eclk_idx}"),
                    )
                    .input_xlat_rc_io(
                        bcls::ECLKBRIDGECS::CLK[0],
                        format!("JCLK0_ECLKBRIDGECS{eclk_idx}"),
                    )
                    .input_xlat_rc_io(
                        bcls::ECLKBRIDGECS::CLK[1],
                        format!("JCLK1_ECLKBRIDGECS{eclk_idx}"),
                    )
                    .output_bind_rc_io(
                        bcls::ECLKBRIDGECS::ECSOUT,
                        format!("ECSOUT_ECLKBRIDGECS{eclk_idx}"),
                        wires::OUT_ECLKBRIDGECS_ECSOUT[i].cell(5),
                    );

                let bcrd_brg = cell.bel(bslots::BRGECLKSYNC[i]);
                self.extract_bel(bcrd_brg)
                    .name(format!("BRGECLKSYNC{eclk_idx}"))
                    .input_xlat_rc_io(bcls::ECLKSYNC::STOP, format!("JSTOP_BRGECLKSYNC{eclk_idx}"))
                    .input_xlat_rc_io(
                        bcls::ECLKSYNC::ECLKI,
                        format!("ECLKI_BRGECLKSYNC{eclk_idx}"),
                    )
                    .output_bind_rc_io(
                        bcls::ECLKSYNC::ECLKO,
                        format!("JECLKO_BRGECLKSYNC{eclk_idx}"),
                        wires::OUT_BRGECLKSYNC_ECLKO[i].cell(5),
                    )
                    .claim_pip("ECLKO", "ECLKI");
            }
        }
    }

    pub(super) fn process_ddrdll_ecp4(&mut self) {
        for (cols, row, ncol, is_corner, name, eclk_banks) in [
            (
                [self.chip.col_w() + 2, self.chip.col_w() + 3],
                self.chip.row_s(),
                self.chip.col_w(),
                false,
                "DDRDLL_BL",
                [6].as_slice(),
            ),
            (
                [self.chip.col_e() - 3, self.chip.col_e() - 2],
                self.chip.row_s(),
                self.chip.col_e(),
                false,
                "DDRDLL_BR",
                [5].as_slice(),
            ),
            (
                [self.chip.col_w() + 2, self.chip.col_w() + 3],
                self.chip.row_n(),
                self.chip.col_w(),
                true,
                "DDRDLL_TL",
                [7, 1].as_slice(),
            ),
            (
                [self.chip.col_e() - 3, self.chip.col_e() - 2],
                self.chip.row_n(),
                self.chip.col_e(),
                true,
                "DDRDLL_TR",
                [2, 4].as_slice(),
            ),
        ] {
            for idx in 0..2 {
                let bcrd = Chip::DIE.cell(cols[idx], row).bel(bslots::DDRDLL);
                let cell = Chip::DIE.cell(ncol, row);
                let mode = if is_corner {
                    RcMode::Corner
                } else {
                    RcMode::Io
                };
                {
                    let clk = self.rc_wire_mode(cell, &format!("JCLK_DDRDLL{idx}"), mode);
                    let clk_in = self.find_single_in(clk);
                    let mut ex = self.extract_switchbox(bcrd.bel(bslots::BEL_INT));
                    ex.cell(cell)
                        .output_bind(clk_in, wires::IMUX_DDRDLL_CLK.cell(0))
                        .input_xlat_rc_mode(format!("JCIBCLK{idx}"), mode);
                    for &bank in eclk_banks {
                        for i in 0..4 {
                            let bcrd_eclk = ex.ctx.chip.bel_eclksync_bank(bank, i);
                            let wire_eclk = ex.ctx.naming.bel_wire(bcrd_eclk, "ECLK");
                            ex.input_xlat(wire_eclk);
                        }
                    }
                }

                let mut bel = self.extract_bel_cell(bcrd, cell);
                bel.name(format!("{name}{idx}"))
                    .input_xlat_rc_mode(bcls::DDRDLL::CLK, format!("JCLK_DDRDLL{idx}"), mode)
                    .input_xlat_rc_mode(bcls::DDRDLL::RST, format!("JRST_DDRDLL{idx}"), mode)
                    .input_xlat_rc_mode(
                        bcls::DDRDLL::UDDCNTLN,
                        format!("JUDDCNTLN_DDRDLL{idx}"),
                        mode,
                    )
                    .input_xlat_rc_mode(bcls::DDRDLL::FREEZE, format!("JFREEZE_DDRDLL{idx}"), mode)
                    .output_xlat_rc_mode(bcls::DDRDLL::LOCK, format!("JLOCK_DDRDLL{idx}"), mode)
                    .output_xlat_rc_mode(bcls::DDRDLL::DIVOSC, format!("JDIVOSC_DDRDLL{idx}"), mode)
                    .output_bind_step_rc_mode(
                        bcls::DDRDLL::DDRDEL,
                        format!("DDRDEL_DDRDLL{idx}"),
                        mode,
                        wires::OUT_DDRDLL_DDRDEL.cell(0),
                    );
                for i in 0..8 {
                    bel.output_xlat_rc_mode(
                        bcls::DDRDLL::DCNTL[i],
                        format!("JDCNTL{i}_DDRDLL{idx}"),
                        mode,
                    );
                }
            }
        }
    }

    pub(super) fn process_dlldel_ecp4(&mut self) {
        let chip = self.chip;
        for (name, edge, idx) in [
            ("00", Dir::N, 0),
            ("01", Dir::N, 1),
            ("10", Dir::N, 2),
            ("11", Dir::N, 3),
            ("20", Dir::N, 4),
            ("21", Dir::N, 5),
            ("30", Dir::N, 6),
            ("31", Dir::N, 7),
            ("60", Dir::W, 0),
            ("61", Dir::W, 1),
            ("70", Dir::W, 2),
            ("71", Dir::W, 3),
            ("50", Dir::E, 0),
            ("51", Dir::E, 1),
            ("40", Dir::E, 2),
            ("41", Dir::E, 3),
        ] {
            if !self
                .chip
                .special_io
                .contains_key(&SpecialIoKey::Clock(edge, idx as u8))
            {
                continue;
            }
            let cell_tile = self.chip.clk_emid(edge);
            let cell = match edge {
                Dir::N => cell_tile.delta(-4 + (idx as i32), 0),
                Dir::W => cell_tile.delta(0, -2 + (idx as i32)),
                Dir::E => cell_tile.delta(0, -2 + ((idx ^ 1) as i32)),
                Dir::S => unreachable!(),
            };

            let bcrd = cell_tile.bel(bslots::DLLDEL[idx]);

            {
                let cslot = self.edev[self.edev.bel_tile(bcrd)]
                    .cells
                    .iter()
                    .find(|&(_, &tcell)| tcell == cell)
                    .unwrap()
                    .0;
                let mut ex = self.extract_switchbox(cell_tile.bel(bslots::CLK_INT));
                ex.cell(cell)
                    .output_bind_rc_io("DDRDEL", cslot.wire(wires::IMUX_DLLDEL_DDRDEL))
                    .output_xlat_rc_io("JINCK")
                    .input_xlat_rc_io("JPADDI")
                    .input_bind_rc_io("DLLDEL", cslot.wire(wires::OUT_DLLDEL_Z));
                let ddrdll_sources = match edge {
                    Dir::W => [
                        (chip.col_w() + 2, chip.row_s()),
                        (chip.col_w() + 3, chip.row_s()),
                        (chip.col_w() + 2, chip.row_n()),
                        (chip.col_w() + 3, chip.row_n()),
                    ],
                    Dir::E => [
                        (chip.col_e() - 3, chip.row_s()),
                        (chip.col_e() - 2, chip.row_s()),
                        (chip.col_e() - 3, chip.row_n()),
                        (chip.col_e() - 2, chip.row_n()),
                    ],
                    Dir::N => [
                        (chip.col_w() + 2, chip.row_n()),
                        (chip.col_w() + 3, chip.row_n()),
                        (chip.col_e() - 3, chip.row_n()),
                        (chip.col_e() - 2, chip.row_n()),
                    ],
                    Dir::S => unreachable!(),
                };
                for (col, row) in ddrdll_sources {
                    let bcrd_ddrdll = Chip::DIE.cell(col, row).bel(bslots::DDRDLL);
                    let wire_ddrdll = ex.ctx.naming.bel_wire(bcrd_ddrdll, "DDRDEL_OUT");
                    ex.input_xlat(wire_ddrdll);
                }
            }

            self.extract_bel_cell(bcrd, cell)
                .name(format!("DLLDEL_{name}"))
                .suffix("DLLDEL")
                .input_xlat_rc_io(bcls::DLLDEL::DDRDEL, "DDRDEL_DLLDEL")
                .output_xlat_rc_io(bcls::DLLDEL::Z, "Z_DLLDEL");
        }
    }

    pub(super) fn process_dtr_ecp4(&mut self) {
        for (tcid, name) in [(tcls::DTR_S, "DTR_BR"), (tcls::DTR_N, "DTR_TL")] {
            for &tcrd in &self.edev.tile_index[tcid] {
                self.extract_bel(tcrd.bel(bslots::DTR))
                    .name(name)
                    .suffix("DTR");
            }
        }
    }

    pub fn xlat_io_loc_ecp4(&self, bcrd: BelCoord) -> (CellCoord, &'static str) {
        let idx = bslots::IO.index_of(bcrd.slot).unwrap();
        let tcrd = self.edev.bel_tile(bcrd);
        let cell = self.edev[tcrd].cells[CellSlotId::from_idx(idx)];
        if let Dir::H(edge) = self.chip.get_io_edge(bcrd.cell)
            && cell.col != self.chip.col_edge(edge)
        {
            let abcd = match self.chip.rows[cell.row].kind {
                RowKind::Ebr => ["EA", "EB", "EC", "ED"][idx],
                RowKind::Dsp => ["EA", "EB"][idx % 2],
                _ => unreachable!(),
            };
            (cell.with_col(self.chip.col_edge(edge)), abcd)
        } else {
            (cell, "")
        }
    }

    fn process_dqs_ecp4(&mut self, bcrd: BelCoord) {
        let chip = self.chip;
        let bcrd_io = bcrd.bel(bslots::IO[0]);
        let edge = chip.get_io_edge(bcrd.cell);
        let bank = chip.get_io_bank(bcrd.cell);
        let (cell, abcd) = self.xlat_io_loc_ecp4(bcrd_io);
        let cslot = self.edev[self.edev.bel_tile(bcrd)]
            .cells
            .iter()
            .find(|&(_, &cell)| cell == bcrd.cell)
            .unwrap()
            .0;

        let (r, c) = self.rc(cell);
        let name = match edge {
            Dir::W => format!("LDQS{r}"),
            Dir::E => format!("RDQS{r}"),
            Dir::N => format!("TDQS{c}"),
            Dir::S => unreachable!(),
        };

        {
            let mut ex = self.extract_switchbox(bcrd.bel(bslots::BEL_INT));
            ex.cell(cell)
                .output_bind_rc_io("DDRDEL", wires::IMUX_DQS_DDRDEL.cell(0))
                .output_bind_rc_io("DQSECLK", wires::IMUX_DQS_ECLK.cell(0))
                .output_bind_rc_io("ECLK90", wires::IMUX_DQS_ECLK90.cell(0));

            let ddrdll_sources = match edge {
                Dir::W => [
                    (chip.col_w() + 2, chip.row_s()),
                    (chip.col_w() + 3, chip.row_s()),
                    (chip.col_w() + 2, chip.row_n()),
                    (chip.col_w() + 3, chip.row_n()),
                ],
                Dir::E => [
                    (chip.col_e() - 3, chip.row_s()),
                    (chip.col_e() - 2, chip.row_s()),
                    (chip.col_e() - 3, chip.row_n()),
                    (chip.col_e() - 2, chip.row_n()),
                ],
                Dir::N => [
                    (chip.col_w() + 2, chip.row_n()),
                    (chip.col_w() + 3, chip.row_n()),
                    (chip.col_e() - 3, chip.row_n()),
                    (chip.col_e() - 2, chip.row_n()),
                ],
                Dir::S => unreachable!(),
            };
            for (col, row) in ddrdll_sources {
                let bcrd_ddrdll = cell.with_cr(col, row).bel(bslots::DDRDLL);
                let wire_ddrdll = ex.ctx.naming.bel_wire(bcrd_ddrdll, "DDRDEL_OUT");
                ex.input_xlat(wire_ddrdll);
            }

            for i in 0..4 {
                let bcrd_eclk = chip.bel_eclksync_bank(bank, i);
                let wire_eclk = ex.ctx.naming.bel_wire(bcrd_eclk, "ECLK");
                ex.input_xlat(wire_eclk);
            }
        }

        let paddi = self.rc_io_wire(cell, &format!("JPADDI{abcd}_PIO"));

        let nextclk = self.rc_io_wire(cell, "NEXTCLK_DQS");
        let nextclk_out = self.find_single_out(nextclk);
        self.bind_wire(nextclk_out, bcrd.wire(wires::OUT_DQS_NEXTCLK));

        self.hybrid_out_rc_io_cell(bcrd, cell, "JCRUCLK_DQS", cslot.wire(wires::OUT_DQS_CRUCLK));
        self.hybrid_out_rc_io_cell(bcrd, cell, "JDIVCLK_DQS", cslot.wire(wires::OUT_DQS_DIVCLK));

        let mut bel = self.extract_bel_cell(bcrd, cell);
        bel.name(name)
            .suffix("DQS")
            .add_wire_rc_io("DQSI", "JDQSI_DQS")
            .claim_pip_from("DQSI", paddi)
            .input_xlat_rc_io(bcls::DQS::DDRDEL, "DDRDEL_DQS")
            .input_xlat_rc_io(bcls::DQS::ECLK, "ECLK_DQS")
            .input_xlat_rc_io(bcls::DQS::ECLK90, "ECLK90_DQS")
            .output_xlat_rc_io(bcls::DQS::NEXTCLK, "NEXTCLK_DQS")
            .add_wire("NEXTCLK_OUT", nextclk_out);

        for pin in [
            "RDPNTR0", "RDPNTR1", "RDPNTR2", "WRPNTR0", "WRPNTR1", "WRPNTR2",
        ] {
            let wire = bel.add_wire_rc_io_get(pin, format!("{pin}_DQS"));
            let wire_out = bel.ctx.claim_single_out(wire);
            bel.add_wire(format!("{pin}_OUT"), wire_out);
        }
        for pin in ["CRUCLK", "DQSR90", "DQSW270", "DQSW"] {
            let wire = bel.ctx.rc_io_wire(cell, &format!("J{pin}_DQS"));
            let wire_out = bel.ctx.pips_fwd[&wire]
                .iter()
                .copied()
                .find(|w| !bel.ctx.int_wires.contains_key(w))
                .unwrap();
            bel.add_wire(format!("{pin}_OUT"), wire_out);
            bel.ctx.claim_pip(wire_out, wire);
            if pin == "CRUCLK" {
                bel.ctx
                    .bind_wire(wire_out, bcrd.wire(wires::OUT_DQS_CRUCLK));
            }
        }
    }

    fn process_single_io_ecp4(&mut self, bcrd: BelCoord) {
        let edge = self.chip.get_io_edge(bcrd.cell);
        let idx = bslots::IO.index_of(bcrd.slot).unwrap();
        let (cell, abcd) = self.xlat_io_loc_ecp4(bcrd);

        let (r, c) = self.rc(cell);
        let names = match edge {
            Dir::W => [format!("PL{r}{abcd}"), format!("IOL_L{r}{abcd}")],
            Dir::E => [format!("PR{r}{abcd}"), format!("IOL_R{r}{abcd}")],
            Dir::N => [format!("PT{c}"), format!("IOL_T{c}")],
            Dir::S => unreachable!(),
        };
        let cslot = self.edev[self.edev.bel_tile(bcrd)]
            .cells
            .iter()
            .find(|&(_, &cell)| cell == bcrd.cell)
            .unwrap()
            .0;

        let mut bel = self.extract_bel_cell(bcrd, cell);
        bel.names(names)
            .input_xlat_rc_io(bcls::IO::CLK, format!("JCLK{abcd}_IOLOGIC"))
            .input_xlat_rc_io(bcls::IO::LSR, format!("JLSR{abcd}_IOLOGIC"))
            .input_xlat_rc_io(bcls::IO::CE, format!("JCE{abcd}_IOLOGIC"))
            .input_xlat_rc_io(bcls::IO::ECLK, format!("ECLK{abcd}_IOLOGIC"))
            .input_xlat_rc_io(bcls::IO::LOADN, format!("JLOADN{abcd}_IOLOGIC"))
            .input_xlat_rc_io(bcls::IO::MOVE, format!("JMOVE{abcd}_IOLOGIC"))
            .input_xlat_rc_io(bcls::IO::DIRECTION, format!("JDIRECTION{abcd}_IOLOGIC"))
            .input_xlat_rc_io(bcls::IO::SLIP, format!("JSLIP{abcd}_IOLOGIC"))
            .input_xlat_rc_io(bcls::IO::ACK, format!("JACK{abcd}_IOLOGIC"))
            .input_xlat_rc_io(
                bcls::IO::WINDOWSIZE[0],
                format!("JWINDOWSIZE0{abcd}_IOLOGIC"),
            )
            .input_xlat_rc_io(
                bcls::IO::WINDOWSIZE[1],
                format!("JWINDOWSIZE1{abcd}_IOLOGIC"),
            )
            .output_xlat_rc_io(bcls::IO::PLUS, format!("JPLUS{abcd}_IOLOGIC"))
            .output_xlat_rc_io(bcls::IO::MINUS, format!("JMINUS{abcd}_IOLOGIC"))
            .output_xlat_rc_io(bcls::IO::CFLAG, format!("JCFLAG{abcd}_IOLOGIC"));
        for i in 0..4 {
            bel.input_xlat_rc_io(bcls::IO::TSDATA[i], format!("JTSDATA{i}{abcd}_IOLOGIC"));
        }
        for i in 0..10 {
            bel.input_xlat_rc_io(bcls::IO::TXDATA[i], format!("JTXDATA{i}{abcd}_IOLOGIC"))
                .output_xlat_rc_io(bcls::IO::RXDATA[i], format!("JRXDATA{i}{abcd}_IOLOGIC"));
        }

        bel.add_wire_rc_io("PADDI_PIO", format!("JPADDI{abcd}_PIO"))
            .add_wire_rc_io("PADDI_IOLOGIC", format!("JPADDI{abcd}_IOLOGIC"))
            .claim_pip("PADDI_IOLOGIC", "PADDI_PIO")
            .add_wire_rc_io("INDD_IOLOGIC", format!("INDD{abcd}_IOLOGIC"))
            .output_xlat_rc_io(bcls::IO::DI, format!("JDI{abcd}"))
            .claim_pip("DI", "PADDI_PIO")
            .claim_pip("DI", "INDD_IOLOGIC")
            .add_wire_rc_io("DI_IOLOGIC", format!("DI{abcd}_IOLOGIC"))
            .claim_pip("DI_IOLOGIC", "DI");

        if idx == 3 {
            let bcrd2 = bcrd.bel(bslots::IO[2]);
            let (cell2, abcd2) = bel.ctx.xlat_io_loc_ecp4(bcrd2);
            let paddi_pio2 = bel.ctx.rc_io_wire(cell2, &format!("JPADDI{abcd2}_PIO"));
            bel.claim_pip_from("PADDI_IOLOGIC", paddi_pio2)
                .claim_pip_from("DI", paddi_pio2);
        }

        bel.output_xlat_rename_rc_io(bcls::IO::RXDATA[0], "INFF", format!("JINFF{abcd}_IOLOGIC"))
            .input_xlat_rename_rc_io(bcls::IO::TXDATA[0], "PADDO", format!("JPADDO{abcd}"))
            .add_wire_rc_io("PADDO_PIO", format!("PADDO{abcd}_PIO"))
            .claim_pip("PADDO_PIO", "PADDO")
            .input_xlat_rename_rc_io(bcls::IO::TSDATA[0], "PADDT", format!("JPADDT{abcd}"))
            .add_wire_rc_io("PADDT_PIO", format!("PADDT{abcd}_PIO"))
            .claim_pip("PADDT_PIO", "PADDT");

        bel.add_wire_rc_io("IOLDO_IOLOGIC", format!("IOLDO{abcd}_IOLOGIC"))
            .add_wire_rc_io("IOLDOI_IOLOGIC", format!("IOLDOI{abcd}_IOLOGIC"))
            .claim_pip("IOLDOI_IOLOGIC", "IOLDO_IOLOGIC")
            .add_wire_rc_io("IOLDOD_IOLOGIC", format!("IOLDOD{abcd}_IOLOGIC"))
            .add_wire_rc_io("IOLDO", format!("IOLDO{abcd}"))
            .claim_pip("IOLDO", "IOLDO_IOLOGIC")
            .claim_pip("IOLDO", "IOLDOD_IOLOGIC")
            .add_wire_rc_io("IOLDO_PIO", format!("IOLDO{abcd}_PIO"))
            .claim_pip("IOLDO_PIO", "IOLDO")
            .add_wire_rc_io("IOLTO_IOLOGIC", format!("IOLTO{abcd}_IOLOGIC"))
            .add_wire_rc_io("IOLTO_PIO", format!("IOLTO{abcd}_PIO"))
            .claim_pip("IOLTO_PIO", "IOLTO_IOLOGIC")
            .output(bcls::IO::INCK, cslot.wire(wires::OUT_IO_INCK[idx]));

        let bank = bel.ctx.chip.get_io_bank(bcrd.cell);
        let has_fucked_pusl = edge == Dir::W
            && abcd.is_empty()
            && if cell.row < bel.ctx.chip.row_clk {
                bel.ctx.chip.rows[cell.row].kind == RowKind::Ebr
            } else {
                bel.ctx.chip.rows[cell.row].kind == RowKind::Dsp
            };
        for (pin, bslot_rc, pin_src) in [
            ("PG", bslots::BCPG, "PGENO_OUT"),
            ("INRD", bslots::BCINRD, "INRDENO_OUT"),
            ("LVDS", bslots::BCLVDSO, "LVDSENO_OUT"),
            ("PUSL", bslots::BCPUSL, "PUSLENO_OUT"),
        ] {
            let wire = if has_fucked_pusl && pin == "PUSL" {
                // what in *fuck's* name
                bel.ctx.rc_io_wire(cell, &format!("LVDS{abcd}_PIO"))
            } else {
                bel.add_wire_rc_io_get(pin, format!("{pin}{abcd}_PIO"))
            };
            let cell_rc = bel.ctx.chip.special_loc[&SpecialLocKey::Bc(bank)];
            let bcrd_rc = cell_rc.bel(bslot_rc);
            let wire_src = bel.ctx.naming.bel_wire(bcrd_rc, pin_src);
            bel.ctx.claim_pip(wire, wire_src);
        }

        for pin in [
            "DQSW", "DQSR90", "DQSW270", "RDPNTR0", "RDPNTR1", "RDPNTR2", "WRPNTR0", "WRPNTR1",
            "WRPNTR2",
        ] {
            if bcrd.col == bel.ctx.chip.col_e() - 1
                && let Some(idx) = pin.strip_prefix("RDPNTR")
            {
                // fuck's sake fuck's sake fuck's sake
                bel.add_wire_rc_io(pin, format!("REPNTR{idx}{abcd}_IOLOGIC"));
            } else {
                bel.add_wire_rc_io(pin, format!("{pin}{abcd}_IOLOGIC"));
            }
            if let Some(bcrd_dqs) = bel.ctx.edev.bel_io_dqs(bcrd) {
                let wire_dqs = bel.ctx.naming.bel_wire(bcrd_dqs, &format!("{pin}_OUT"));
                bel.claim_pip_from(pin, wire_dqs);
            }
        }
    }

    pub(super) fn process_io_ecp4(&mut self) {
        for tcid in [
            tcls::DQS_W,
            tcls::DQS_W_BELOW_DSP_N,
            tcls::DQS_W_BELOW_EBR_N,
            tcls::DQS_W_BELOW_EBR_S,
            tcls::DQS_W_EBR_S,
            tcls::DQS_W_EBR_N,
            tcls::DQS_W_DSP_N,
            tcls::DQS_E,
            tcls::DQS_E_BELOW_DSP_N,
            tcls::DQS_E_BELOW_EBR_N,
            tcls::DQS_E_BELOW_EBR_S,
            tcls::DQS_E_EBR_S,
            tcls::DQS_E_EBR_N,
            tcls::DQS_E_DSP_N,
            tcls::DQS_N,
        ] {
            for &tcrd in &self.edev.tile_index[tcid] {
                let bcrd = tcrd.bel(bslots::DQS[0]);
                self.process_dqs_ecp4(bcrd);
            }
        }
        for tcid in [
            tcls::IO_W,
            tcls::IO_W_EBR_S,
            tcls::IO_W_EBR_N,
            tcls::IO_W_DSP_S,
            tcls::IO_W_DSP_N,
            tcls::IO_E,
            tcls::IO_E_EBR_S,
            tcls::IO_E_EBR_N,
            tcls::IO_E_DSP_S,
            tcls::IO_E_DSP_N,
            tcls::IO_N,
        ] {
            for &tcrd in &self.edev.tile_index[tcid] {
                let cslot = self.edev[tcrd]
                    .cells
                    .iter()
                    .find(|&(_, &cell)| cell == tcrd.cell)
                    .unwrap()
                    .0;
                let io0 = tcrd.bel(bslots::IO[0]);
                let bank = self.chip.get_io_bank(tcrd.cell);
                {
                    let mut ex = self.extract_switchbox(tcrd.bel(bslots::IO_INT));
                    for i in 0..4 {
                        let bcrd_io = tcrd.bel(bslots::IO[i]);
                        let (cell_io, abcd) = ex.ctx.xlat_io_loc_ecp4(bcrd_io);
                        let eclk = ex.ctx.rc_io_wire(cell_io, &format!("ECLK{abcd}"));
                        ex.output_bind(eclk, cslot.wire(wires::IMUX_IO_ECLK[i]));
                    }
                    for i in 0..4 {
                        let bcrd_eclk = ex.ctx.chip.bel_eclksync_bank(bank, i);
                        let wire_eclk = ex.ctx.naming.bel_wire(bcrd_eclk, "ECLK");
                        ex.input_xlat(wire_eclk);
                    }
                    if let Some(bcrd_dqs) = ex.ctx.edev.bel_io_dqs(io0) {
                        let nextclk = ex.ctx.naming.bel_wire(bcrd_dqs, "NEXTCLK_OUT");
                        let cruclk = ex.ctx.naming.bel_wire(bcrd_dqs, "CRUCLK_OUT");
                        ex.input_xlat(nextclk).input_xlat(cruclk);
                    } else {
                        ex.missing_input(cslot.wire(wires::OUT_DQS_NEXTCLK))
                            .missing_input(cslot.wire(wires::OUT_DQS_CRUCLK));
                    }
                }
                for i in 0..4 {
                    let bcrd = tcrd.bel(bslots::IO[i]);
                    self.process_single_io_ecp4(bcrd);
                }
            }
        }
    }

    pub(super) fn process_clkdiv_ecp4(&mut self) {
        for (lrt, edge) in [('T', Dir::N), ('L', Dir::W), ('R', Dir::E)] {
            let cell_tile = self.chip.clk_emid(edge);
            let cell = if edge == Dir::N {
                cell_tile.step(Dir::W, 1)
            } else {
                cell_tile
            };
            let bcrd_int = cell_tile.bel(bslots::CLK_INT);
            let ci_base = if self.edev[self.edev.bel_tile(bcrd_int)].class == tcls::CLK_N_L {
                9
            } else {
                5
            };
            {
                let mut ex = self.extract_switchbox(bcrd_int);
                ex.cell(cell)
                    .output_bind_rc_io("CLKI0", wires::IMUX_CLKDIV_CLKI[0].cell(ci_base))
                    .output_bind_rc_io("CLKI1", wires::IMUX_CLKDIV_CLKI[1].cell(ci_base))
                    .output_bind_rc_io("CLKI2", wires::IMUX_CLKDIV_CLKI[2].cell(ci_base))
                    .output_bind_rc_io("CLKI3", wires::IMUX_CLKDIV_CLKI[3].cell(ci_base));
                for i in 0..4 {
                    for bank_idx in 0..4 {
                        let bcrd_eclk = bcrd_int.bel(bslots::ECLKSYNC[bank_idx * 4 + i]);
                        if !ex.ctx.edev.has_bel(bcrd_eclk) {
                            continue;
                        }
                        let wire_eclk = ex.ctx.naming.bel_wire(bcrd_eclk, "ECLK");
                        ex.input_xlat(wire_eclk);
                    }
                }
            }
            for i in 0..4 {
                let bcrd = cell_tile.bel(bslots::CLKDIV[i]);
                self.hybrid_out_rc_io_cell(
                    bcrd,
                    cell,
                    format!("JCDIVX_CLKDIV{i}"),
                    wires::OUT_CLKDIV_CDIVX[i].cell(ci_base),
                );

                self.extract_bel_cell(bcrd, cell)
                    .name(format!("CLKDIV_{lrt}{i}"))
                    .input_xlat_rc_io(bcls::CLKDIV::CLKI, format!("CLKI_CLKDIV{i}"))
                    .input_xlat_rc_io(bcls::CLKDIV::RST, format!("JRST_CLKDIV{i}"))
                    .input_xlat_rc_io(bcls::CLKDIV::ALIGNWD, format!("JALIGNWD_CLKDIV{i}"))
                    .output_xlat_rc_io(bcls::CLKDIV::CDIVX, format!("JCDIVX_CLKDIV{i}"));
            }
        }
    }

    pub(super) fn process_pcsclkdiv_ecp4(&mut self) {
        let num_quads = if self
            .chip
            .special_loc
            .contains_key(&SpecialLocKey::SerdesSingle)
        {
            1
        } else if self
            .chip
            .special_loc
            .contains_key(&SpecialLocKey::SerdesDouble)
        {
            2
        } else if self
            .chip
            .special_loc
            .contains_key(&SpecialLocKey::SerdesTriple)
        {
            3
        } else {
            unreachable!()
        };

        let cell_tile = self.chip.clk_emid(Dir::S);
        let cell = cell_tile.delta(-1, 0);
        for i in 0..4 {
            if i < num_quads {
                let abcd = ['A', 'B', 'C', 'D'][i];
                let mut ex = self.extract_switchbox(cell_tile.bel(bslots::CLK_INT));
                ex.cell(cell)
                    .output_bind_rc(format!("PCSCDIVI{i}"), wires::IMUX_CLKDIV_CLKI[i].cell(1))
                    .input_xlat_rc(format!("JPCSCDIVCIB{i}"));
                for j in 0..4 {
                    ex.input_xlat_rc(format!("JPCS{abcd}TXCLK{j}"))
                        .input_xlat_rc(format!("JPCS{abcd}RXCLK{j}"));
                }
            }
            let bcrd = cell_tile.bel(bslots::PCSCLKDIV[i]);
            let mut bel = self.extract_bel_cell(bcrd, cell);
            bel.name(format!("PCSCLKDIV{i}"))
                .output_bind_rc(
                    bcls::PCSCLKDIV::CDIV1,
                    format!("CDIV1_PCSCLKDIV{i}"),
                    wires::OUT_CLKDIV_CDIV1[i].cell(1),
                )
                .output_bind_rc(
                    bcls::PCSCLKDIV::CDIVX,
                    format!("CDIVX_PCSCLKDIV{i}"),
                    wires::OUT_CLKDIV_CDIVX[i].cell(1),
                );

            if i < num_quads {
                bel.input_xlat_rc(bcls::PCSCLKDIV::CLKI, format!("CLKI_PCSCLKDIV{i}"))
                    .input_xlat_rc(bcls::PCSCLKDIV::RST, format!("JRST_PCSCLKDIV{i}"))
                    .input_xlat_rc(bcls::PCSCLKDIV::SEL[0], format!("JSEL0_PCSCLKDIV{i}"))
                    .input_xlat_rc(bcls::PCSCLKDIV::SEL[1], format!("JSEL1_PCSCLKDIV{i}"))
                    .input_xlat_rc(bcls::PCSCLKDIV::SEL[2], format!("JSEL2_PCSCLKDIV{i}"));
            } else {
                bel.add_wire_rc("CLKI", format!("CLKI_PCSCLKDIV{i}"))
                    .add_wire_rc("CLKI_IN", format!("PCSCDIVI{i}"))
                    .claim_pip("CLKI", "CLKI_IN")
                    .add_wire_rc("RST", format!("JRST_PCSCLKDIV{i}"))
                    .add_wire_rc("SEL0", format!("JSEL0_PCSCLKDIV{i}"))
                    .add_wire_rc("SEL1", format!("JSEL1_PCSCLKDIV{i}"))
                    .add_wire_rc("SEL2", format!("JSEL2_PCSCLKDIV{i}"));
                bel.discard();
            }
        }
    }
}
