use prjcombine_ecp::{
    chip::{Chip, IoGroupKind, IoKind, SpecialLocKey},
    defs::{self, bcls, bslots, tcls, wires},
};
use prjcombine_interconnect::{
    coord::{BelCoord, CellCoord, DieIdExt},
    db::{CellSlotIdExt, WireSlotIdExt},
    dir::{Dir, DirH, DirHV, DirV},
};

use crate::{ChipContext, RcMode};

impl ChipContext<'_> {
    pub(super) fn prep_io_ecp5(&mut self) {
        for cell in self.edev.die_cells(Chip::DIE) {
            for iob in 0..4 {
                let bcrd = cell.bel(bslots::IO[iob]);
                if self.edev.has_bel(bcrd) {
                    let (cell_io, abcd) = self.xlat_io_loc_ecp5(bcrd);
                    let paddi_pio = self.rc_io_wire(cell_io, &format!("JPADDI{abcd}_PIO"));
                    self.bind_wire(paddi_pio, bcrd.wire(wires::OUT_IO_INCK[iob]));
                }
            }
        }
    }

    pub(super) fn process_bc_ecp5(&mut self) {
        for (&key, &cell_loc) in &self.chip.special_loc {
            let SpecialLocKey::Bc(bank) = key else {
                continue;
            };
            let (col, row, suffix) = match bank {
                1 | 2 => (self.chip.col_e(), self.chip.row_n(), "R"),
                3 | 4 => (self.chip.col_e(), self.chip.row_s(), "R"),
                8 | 6 => (self.chip.col_w(), self.chip.row_s(), "L"),
                7 | 0 => (self.chip.col_w(), self.chip.row_n(), "L"),
                _ => unreachable!(),
            };
            let cell = Chip::DIE.cell(col, row);
            for (bel, name, pin_out, pin_in, pname_in) in [
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
            ] {
                let bcrd = cell_loc.bel(bel);
                if !self.edev.has_bel(bcrd) {
                    continue;
                }

                let wire_in = self.rc_corner_wire(cell, &format!("J{pname_in}_{name}_{suffix}"));
                let wire_out = self.rc_corner_wire(cell, &format!("{pin_out}_{name}_{suffix}"));
                let wire_out_out = self.claim_single_out(wire_out);
                self.claim_pip(wire_out, wire_in);

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
                    let wire_src_in = bel
                        .ctx
                        .rc_corner_wire(cell, &format!("JPVT_SRC_IN{i}_BREFTEST{bank}"));
                    let wire_snk_in = bel
                        .ctx
                        .rc_corner_wire(cell, &format!("JPVT_SNK_IN{i}_BREFTEST{bank}"));
                    let wire_src_out = bel
                        .ctx
                        .rc_corner_wire(cell, &format!("JPVT_SRC_OUT{i}_BREFTEST{bank}"));
                    let wire_snk_out = bel
                        .ctx
                        .rc_corner_wire(cell, &format!("JPVT_SNK_OUT{i}_BREFTEST{bank}"));
                    bel.input_xlat(bcls::BREFTEST::PVT_SRC_IN[i], wire_src_in)
                        .input_xlat(bcls::BREFTEST::PVT_SNK_IN[i], wire_snk_in)
                        .output_xlat(bcls::BREFTEST::PVT_SRC_OUT[i], wire_src_out)
                        .output_xlat(bcls::BREFTEST::PVT_SNK_OUT[i], wire_snk_out);
                }
            }
        }
        if !self.chip.special_loc.contains_key(&SpecialLocKey::Bc(4)) {
            self.dummy_sites.insert("BREFTEST4".into());
            let cell = Chip::DIE.cell(self.chip.col_e(), self.chip.row_s());
            for pin in ["PVT_SRC_IN", "PVT_SNK_IN", "PVT_SRC_OUT", "PVT_SNK_OUT"] {
                for i in 0..6 {
                    let wire = self.rc_corner_wire(cell, &format!("J{pin}{i}_BREFTEST4"));
                    self.claim_node(wire);
                }
            }
        }
    }

    pub(super) fn process_eclk_ecp5(&mut self) {
        for edge in DirH::DIRS {
            let cell_tile = self.chip.clk_emid(edge);
            for bank_idx in 0..2 {
                let cell = cell_tile.delta(0, -1 + (bank_idx as i32));
                for eclk_idx in 0..2 {
                    let wname = self.rc_io_wire(cell, &format!("JECLKO_ECLKSYNC{eclk_idx}"));
                    self.claim_node(wname);
                    self.bind_wire(
                        wname,
                        cell_tile.wire(wires::OUT_ECLKSYNC_ECLKO[bank_idx * 2 + eclk_idx]),
                    );
                }
            }
        }
        for (bank, edge, bank_idx) in [
            (6, DirH::W, 0),
            (7, DirH::W, 1),
            (3, DirH::E, 0),
            (2, DirH::E, 1),
        ] {
            let cell_tile = self.chip.clk_emid(edge);
            let lr = match edge {
                DirH::W => 'L',
                DirH::E => 'R',
            };

            let cell = cell_tile.delta(0, -1 + (bank_idx as i32));
            let bcrd_int = cell_tile.bel(bslots::CLK_INT);
            {
                let mut ex = self.extract_switchbox(bcrd_int);
                ex.cell(cell);
                for eclk_idx in 0..2 {
                    let eclki = ex
                        .ctx
                        .rc_io_wire(cell, &format!("ECLKI_ECLKSYNC{eclk_idx}"));
                    let eclki_in = ex.ctx.find_single_in(eclki);
                    ex.output_bind(
                        eclki_in,
                        wires::IMUX_ECLKSYNC_ECLKI[bank_idx * 2 + eclk_idx].cell(6),
                    )
                    .input_xlat_rc_io(format!("JL{lr}QECLKCIB{eclk_idx}"))
                    .input_xlat_rc_io(format!("JU{lr}QECLKCIB{eclk_idx}"))
                    .input_xlat_rc_io(format!("JBRGECLK{eclk_idx}"))
                    .input_xlat_rc_io(format!("SYNCECLK{eclk_idx}"))
                    .input_xlat_rc_io(format!("JNEIGHBORECLK{eclk_idx}"));
                    let wire = wires::ECLK[eclk_idx].cell(5 + bank_idx);
                    let wname = ex.ctx.rc_io_wire(cell, &format!("JECLK{eclk_idx}"));
                    let outs = ex.ctx.xlat_output(ex.bcrd, wname);
                    assert_eq!(outs.len(), 1);
                    let out = outs.into_iter().next().unwrap();
                    ex.permabuf(out, wire).output_bind(wname, wire);
                }
                for i in 0..4 {
                    let name = match edge {
                        DirH::W => ["60", "61", "70", "71"][i],
                        DirH::E => ["30", "31", "20", "21"][i],
                    };
                    ex.input_xlat_rc_io(format!("JPCLKT{name}"));
                }
                for v in DirV::DIRS {
                    let hv = edge & v;
                    if ex.ctx.chip.try_bel_pll(hv, 0).is_some() {
                        let ul = match v {
                            DirV::S => "L",
                            DirV::N => "U",
                        };
                        ex.input_xlat_rc_io(format!("J{ul}{lr}CPLL0CLKOP"))
                            .input_xlat_rc_io(format!("J{ul}{lr}CPLL0CLKOS"));
                    }
                }
            }
            for eclk_idx in 0..2 {
                let bcrd = cell_tile.bel(bslots::ECLKSYNC[bank_idx * 2 + eclk_idx]);

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
                    .input_xlat_rc_io(bcls::ECLKSYNC::ECLKI, format!("ECLKI_ECLKSYNC{eclk_idx}"))
                    .output_xlat_rc_io(bcls::ECLKSYNC::ECLKO, format!("JECLKO_ECLKSYNC{eclk_idx}"))
                    .add_wire("ECLK", eclk)
                    .claim_pip("ECLKO", "ECLKI");
            }
        }

        for (lr, edge) in [('L', Dir::W), ('R', Dir::E)] {
            let bcrd = self.chip.clk_emid(edge).bel(bslots::CLKTEST_ECLK);

            let mut bel = self.extract_bel(bcrd);
            bel.name(format!("CLKTEST_{lr}ECLK"));

            for i in 0..24 {
                if i < 6 {
                    bel.input_xlat_rc_io(bcls::CLKTEST::TESTIN[i], format!("JTESTIN{i}_CLKTEST"));
                } else {
                    bel.add_wire_rc_io(format!("TESTIN{i}"), format!("JTESTIN{i}_CLKTEST"));
                }
            }
        }

        for (edge, eclk_idx) in [(Dir::W, 1), (Dir::E, 0)] {
            let cell = self.chip.clk_emid(edge);
            let bcrd_cs = cell.bel(bslots::ECLKBRIDGECS[0]);

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
                    wires::OUT_ECLKBRIDGECS_ECSOUT[0].cell(6),
                );

            let bcrd_brg = cell.bel(bslots::BRGECLKSYNC[0]);
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
                    wires::OUT_BRGECLKSYNC_ECLKO[0].cell(6),
                )
                .claim_pip("ECLKO", "ECLKI");
        }
    }

    pub(super) fn process_ddrdll_ecp5(&mut self) {
        let chip = self.chip;
        for hv in DirHV::DIRS {
            let cell = chip.special_loc[&SpecialLocKey::DdrDll(hv)];
            let bcrd = cell.bel(bslots::DDRDLL);
            let corner = match hv {
                DirHV::SW => "BL",
                DirHV::SE => "BR",
                DirHV::NW => "TL",
                DirHV::NE => "TR",
            };
            let cell = cell.with_cr(chip.col_edge(hv.h), chip.row_edge(hv.v));

            {
                let clk = self.rc_corner_wire(cell, "JCLK_DDRDLL");
                let clk_in = self.find_single_in(clk);
                let mut ex = self.extract_switchbox(bcrd.bel(bslots::BEL_INT));
                ex.cell(cell)
                    .output_bind(clk_in, wires::IMUX_DDRDLL_CLK.cell(0))
                    .input_xlat_rc_mode("JCIBCLK0", RcMode::Corner);
                let bank = match hv {
                    DirHV::SW => 6,
                    DirHV::SE => 3,
                    DirHV::NW => 7,
                    DirHV::NE => 2,
                };
                for i in 0..2 {
                    let bcrd_eclk = chip.bel_eclksync_bank(bank, i);
                    let wire_eclk = ex.ctx.naming.bel_wire(bcrd_eclk, "ECLK");
                    ex.input_xlat(wire_eclk);
                }
            }

            self.extract_bel_cell(bcrd, cell)
                .name(format!("DDRDLL_{corner}"))
                .suffix("DDRDLL")
                .output_bind_step_rc_mode(
                    bcls::DDRDLL::DDRDEL,
                    "DDRDEL_DDRDLL",
                    RcMode::Corner,
                    wires::OUT_DDRDLL_DDRDEL.cell(0),
                );

            if hv == DirHV::NW {
                let cell = bcrd.cell.with_col(chip.col_w());
                let wire = self.rc_wire(cell, "TESTIN_PVTTEST");
                self.claim_node(wire);
                self.dummy_sites.insert("PVTTEST".to_string());
            }
        }
    }

    pub(super) fn process_dlldel_ecp5(&mut self) {
        let chip = self.chip;
        for (name, edge, idx) in [
            ("00", Dir::N, 0),
            ("01", Dir::N, 1),
            ("10", Dir::N, 2),
            ("11", Dir::N, 3),
            ("60", Dir::W, 0),
            ("61", Dir::W, 1),
            ("70", Dir::W, 2),
            ("71", Dir::W, 3),
            ("30", Dir::E, 0),
            ("31", Dir::E, 1),
            ("20", Dir::E, 2),
            ("21", Dir::E, 3),
        ] {
            let cell_tile = self.chip.clk_emid(edge);
            let cell = match edge {
                Dir::N => cell_tile.delta(-2 + (idx as i32), 0),
                Dir::W => cell_tile.delta(0, -2 + ((idx ^ 1) as i32)),
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
                    Dir::W => [DirHV::SW, DirHV::NW],
                    Dir::E => [DirHV::SE, DirHV::NE],
                    Dir::N => [DirHV::NW, DirHV::NE],
                    _ => unreachable!(),
                };
                for hv in ddrdll_sources {
                    let bcrd_ddrdll =
                        chip.special_loc[&SpecialLocKey::DdrDll(hv)].bel(bslots::DDRDLL);
                    let wire_ddrdll = ex.ctx.naming.bel_wire(bcrd_ddrdll, "DDRDEL_OUT");
                    if edge == Dir::N {
                        let cslot = match hv.h {
                            DirH::W => defs::ecp5::tcls::CLK_N::DDRDLL_NW,
                            DirH::E => defs::ecp5::tcls::CLK_N::DDRDLL_NE,
                        };
                        ex.input_hint(wire_ddrdll, cslot.wire(wires::OUT_DDRDLL_DDRDEL));
                    } else {
                        ex.input_xlat(wire_ddrdll);
                    }
                }
            }

            self.extract_bel_cell(bcrd, cell)
                .name(format!("DLLDEL_{name}"))
                .suffix("DLLDEL")
                .input_xlat_rc_io(bcls::DLLDEL::DDRDEL, "DDRDEL_DLLDEL")
                .output_xlat_rc_io(bcls::DLLDEL::Z, "Z_DLLDEL");
        }
    }

    pub(super) fn process_dtr_ecp5(&mut self) {
        let tcid = tcls::DTR;
        for &tcrd in &self.edev.tile_index[tcid] {
            let mut bel = self.extract_bel(tcrd.bel(bslots::DTR));
            bel.name("DTR")
                .input_xlat_rc(bcls::DTR::START_PULSE, "JSTARTPULSE_DTR");
            for i in 0..8 {
                bel.output_xlat_rc(bcls::DTR::DTR_OUT[i], format!("JDTROUT{i}_DTR"));
            }
        }
    }

    pub fn xlat_io_loc_ecp5(&self, bcrd: BelCoord) -> (CellCoord, &'static str) {
        let idx = bslots::IO.index_of(bcrd.slot).unwrap();
        let abcd = ["A", "B", "C", "D"][idx];
        match self.chip.get_io_edge(bcrd.cell) {
            Dir::H(_) => (bcrd.cell.delta(0, 2), abcd),
            Dir::V(_) => (bcrd.cell, abcd),
        }
    }

    fn process_dqs_ecp5(&mut self, bcrd: BelCoord) {
        let chip = self.chip;
        let edge = chip.get_io_edge(bcrd.cell);
        let bank = chip.get_io_bank(bcrd.cell);
        let cell = bcrd.cell.delta(0, 2);
        let (r, _c) = self.rc(cell);

        let name = match edge {
            Dir::W => format!("LDQS{r}"),
            Dir::E => format!("RDQS{r}"),
            _ => unreachable!(),
        };

        {
            let mut ex = self.extract_switchbox(bcrd.bel(bslots::BEL_INT));
            ex.cell(cell)
                .output_bind_rc_io("DDRDEL", wires::IMUX_DQS_DDRDEL.cell(0))
                .output_bind_rc_io("DQSECLK", wires::IMUX_DQS_ECLK.cell(0));

            let ddrdll_sources = match edge {
                Dir::W => [DirHV::SW, DirHV::NW],
                Dir::E => [DirHV::SE, DirHV::NE],
                _ => unreachable!(),
            };
            for hv in ddrdll_sources {
                let bcrd_ddrdll = chip.special_loc[&SpecialLocKey::DdrDll(hv)].bel(bslots::DDRDLL);
                let wire_ddrdll = ex.ctx.naming.bel_wire(bcrd_ddrdll, "DDRDEL_OUT");
                ex.input_xlat(wire_ddrdll);
            }

            for i in 0..2 {
                let bcrd_eclk = chip.bel_eclksync_bank(bank, i);
                let wire_eclk = ex.ctx.naming.bel_wire(bcrd_eclk, "ECLK");
                ex.input_xlat(wire_eclk);
            }
        }

        let paddi = self.rc_io_wire(cell, "JPADDIA_PIO");

        let mut bel = self.extract_bel_cell(bcrd, cell);
        bel.name(name)
            .suffix("DQS")
            .add_wire_rc_io("DQSI", "JDQSI_DQS")
            .claim_pip_from("DQSI", paddi)
            .input_xlat_rc_io(bcls::DQS::DDRDEL, "DDRDEL_DQS")
            .input_xlat_rc_io(bcls::DQS::ECLK, "ECLK_DQS");

        for pin in [
            "RDPNTR0", "RDPNTR1", "RDPNTR2", "WRPNTR0", "WRPNTR1", "WRPNTR2",
        ] {
            let wire = bel.add_wire_rc_io_get(pin, format!("{pin}_DQS"));
            let wire_out = bel.ctx.claim_single_out(wire);
            bel.add_wire(format!("{pin}_OUT"), wire_out);
        }
        for pin in ["DQSR90", "DQSW270", "DQSW"] {
            let wire = bel.ctx.rc_io_wire(cell, &format!("J{pin}_DQS"));
            let wire_out = bel.ctx.pips_fwd[&wire]
                .iter()
                .copied()
                .find(|w| !bel.ctx.int_wires.contains_key(w))
                .unwrap();
            bel.add_wire(format!("{pin}_OUT"), wire_out);
            bel.ctx.claim_pip(wire_out, wire);
        }
    }

    fn process_single_io_ecp5(&mut self, bcrd: BelCoord) {
        let edge = self.chip.get_io_edge(bcrd.cell);
        let bank = self.chip.get_io_bank(bcrd.cell);
        let idx = bslots::IO.index_of(bcrd.slot).unwrap();
        let cell = match edge {
            Dir::H(_) => bcrd.cell.delta(0, 2),
            Dir::V(_) => bcrd.cell,
        };
        let abcd = ["A", "B", "C", "D"][idx];
        let (r, c) = self.rc(cell);
        let names = match edge {
            Dir::W => [format!("PL{r}{abcd}"), format!("IOL_L{r}{abcd}")],
            Dir::E => [format!("PR{r}{abcd}"), format!("IOL_R{r}{abcd}")],
            Dir::S => [format!("PB{c}{abcd}"), format!("IOL_B{c}{abcd}")],
            Dir::N => [format!("PT{c}{abcd}"), format!("IOL_T{c}{abcd}")],
        };
        let kind = self.chip.get_io_kind(bcrd);
        let iol = match kind {
            IoKind::Io => "IOLOGIC",
            IoKind::Sio => "SIOLOGIC",
            _ => unreachable!(),
        };

        let mut bel = self.extract_bel_cell(bcrd, cell);
        bel.names(names)
            .input_xlat_rc_io(bcls::IO::CLK, format!("JCLK{abcd}_{iol}"))
            .input_xlat_rc_io(bcls::IO::LSR, format!("JLSR{abcd}_{iol}"))
            .input_xlat_rc_io(bcls::IO::CE, format!("JCE{abcd}_{iol}"))
            .input_xlat_rc_io(bcls::IO::LOADN, format!("JLOADN{abcd}_{iol}"))
            .input_xlat_rc_io(bcls::IO::MOVE, format!("JMOVE{abcd}_{iol}"))
            .input_xlat_rc_io(bcls::IO::DIRECTION, format!("JDIRECTION{abcd}_{iol}"))
            .output_xlat_rc_io(bcls::IO::INFF, format!("JINFF{abcd}_{iol}"))
            .output_xlat_rc_io(bcls::IO::CFLAG, format!("JCFLAG{abcd}_{iol}"));

        let (num, num_ts) = if kind == IoKind::Io {
            if matches!(idx, 0 | 2) { (7, 2) } else { (4, 2) }
        } else {
            (2, 1)
        };
        for i in 0..num {
            bel.input_xlat_rc_io(bcls::IO::TXDATA[i], format!("JTXDATA{i}{abcd}_{iol}"))
                .output_xlat_rc_io(bcls::IO::RXDATA[i], format!("JRXDATA{i}{abcd}_{iol}"));
        }
        for i in 0..num_ts {
            bel.input_xlat_rc_io(bcls::IO::TSDATA[i], format!("JTSDATA{i}{abcd}_{iol}"));
        }

        if kind == IoKind::Io {
            bel.input_xlat_rc_io(bcls::IO::SLIP, format!("JSLIP{abcd}_{iol}"))
                .input_xlat_rc_io(bcls::IO::ECLK, format!("ECLK{abcd}_{iol}"));
            if matches!(idx, 1 | 3) {
                for pin in [
                    "RXDATA4", "RXDATA5", "RXDATA6", "TXDATA4", "TXDATA5", "TXDATA6",
                ] {
                    bel.add_wire_rc_io(pin, format!("{pin}{abcd}_{iol}"));
                }
            }
        }

        bel.add_wire_rc_io("PADDI_PIO", format!("JPADDI{abcd}_PIO"))
            .add_wire_rc_io("PADDI_IOLOGIC", format!("PADDI{abcd}_{iol}"))
            .claim_pip("PADDI_IOLOGIC", "PADDI_PIO")
            .add_wire_rc_io("INDD_IOLOGIC", format!("INDD{abcd}_{iol}"))
            .output_xlat_rc_io(bcls::IO::DI, format!("JDI{abcd}"))
            .claim_pip("DI", "PADDI_PIO")
            .claim_pip("DI", "INDD_IOLOGIC")
            .add_wire_rc_io("DI_IOLOGIC", format!("DI{abcd}_{iol}"))
            .claim_pip("DI_IOLOGIC", "DI");

        bel.input_xlat_rename_rc_io(bcls::IO::TXDATA[0], "PADDO", format!("JPADDO{abcd}"))
            .add_wire_rc_io("PADDO_PIO", format!("PADDO{abcd}_PIO"))
            .claim_pip("PADDO_PIO", "PADDO")
            .add_wire_rc_io("PADDT_PIO", format!("PADDT{abcd}_PIO"))
            .input_xlat_rename_rc_io(bcls::IO::TSDATA[0], "PADDT", format!("JPADDT{abcd}"))
            .claim_pip("PADDT_PIO", "PADDT");

        bel.add_wire_rc_io("IOLDO_IOLOGIC", format!("IOLDO{abcd}_{iol}"))
            .add_wire_rc_io("IOLDOI_IOLOGIC", format!("IOLDOI{abcd}_{iol}"))
            .claim_pip("IOLDOI_IOLOGIC", "IOLDO_IOLOGIC")
            .add_wire_rc_io("IOLDOD_IOLOGIC", format!("IOLDOD{abcd}_{iol}"))
            .add_wire_rc_io("IOLDO", format!("IOLDO{abcd}"))
            .claim_pip("IOLDO", "IOLDO_IOLOGIC")
            .claim_pip("IOLDO", "IOLDOD_IOLOGIC")
            .add_wire_rc_io("IOLDO_PIO", format!("IOLDO{abcd}_PIO"))
            .claim_pip("IOLDO_PIO", "IOLDO")
            .add_wire_rc_io("IOLTO_IOLOGIC", format!("IOLTO{abcd}_{iol}"))
            .add_wire_rc_io("IOLTO_PIO", format!("IOLTO{abcd}_PIO"))
            .claim_pip("IOLTO_PIO", "IOLTO_IOLOGIC")
            .output(bcls::IO::INCK, wires::OUT_IO_INCK[idx].cell(0));

        for (pin, bslot_rc, pin_src) in [
            ("INRD", bslots::BCINRD, "INRDENO_OUT"),
            ("LVDS", bslots::BCLVDSO, "LVDSENO_OUT"),
        ] {
            bel.add_wire_rc_io(pin, format!("{pin}{abcd}_PIO"));
            let bcrd_rc = bel.ctx.chip.special_loc[&SpecialLocKey::Bc(bank)].bel(bslot_rc);
            if bel.ctx.edev.has_bel(bcrd_rc) {
                let wire_src = bel.ctx.naming.bel_wire(bcrd_rc, pin_src);
                bel.claim_pip_from(pin, wire_src);
            }
        }

        if kind == IoKind::Io {
            for pin in [
                "DQSW", "DQSR90", "DQSW270", "RDPNTR0", "RDPNTR1", "RDPNTR2", "WRPNTR0", "WRPNTR1",
                "WRPNTR2",
            ] {
                bel.add_wire_rc_io(pin, format!("{pin}{abcd}_IOLOGIC"));
                if let Some(bcrd_dqs) = bel.ctx.edev.bel_io_dqs(bcrd) {
                    let wire_dqs = bel.ctx.naming.bel_wire(bcrd_dqs, &format!("{pin}_OUT"));
                    bel.claim_pip_from(pin, wire_dqs);
                }
            }
        }
    }

    pub(super) fn process_io_ecp5(&mut self) {
        for tcid in [tcls::DQS_W, tcls::DQS_E] {
            for &tcrd in &self.edev.tile_index[tcid] {
                self.process_dqs_ecp5(tcrd.bel(bslots::DQS[0]));
            }
        }
        for (tcid, num_io) in [
            (tcls::IO_W4, 4),
            (tcls::IO_E4, 4),
            (tcls::IO_N2, 2),
            (tcls::IO_S2, 2),
            (tcls::IO_S1, 1),
        ] {
            for &tcrd in &self.edev.tile_index[tcid] {
                if matches!(tcid, tcls::IO_W4 | tcls::IO_E4) {
                    let cell = tcrd.delta(0, 2);
                    let bank = self.chip.get_io_bank(tcrd.cell);
                    let mut ex = self.extract_switchbox(tcrd.bel(bslots::IO_INT));
                    for i in 0..2 {
                        let bcrd_eclk = ex.ctx.chip.bel_eclksync_bank(bank, i);
                        let wire_eclk = ex.ctx.naming.bel_wire(bcrd_eclk, "ECLK");
                        ex.input_xlat(wire_eclk);
                    }
                    ex.cell(cell)
                        .output_bind_rc_io("ECLKA", wires::IMUX_IO_ECLK[0].cell(0))
                        .output_bind_rc_io("ECLKB", wires::IMUX_IO_ECLK[1].cell(0))
                        .output_bind_rc_io("ECLKC", wires::IMUX_IO_ECLK[2].cell(0))
                        .output_bind_rc_io("ECLKD", wires::IMUX_IO_ECLK[3].cell(0));
                }
                for i in 0..num_io {
                    let bcrd = tcrd.bel(bslots::IO[i]);
                    self.process_single_io_ecp5(bcrd);
                }
            }
        }
        if self.skip_serdes && self.chip.rows.len() == 49 {
            // what. have you been drinking.
            let cell = Chip::DIE.cell(self.chip.col_clk - 1, self.chip.row_n());
            for wn in [
                "JCFLAGA_SIOLOGIC",
                "JRXDATA1A_SIOLOGIC",
                "JRXDATA0A_SIOLOGIC",
                "JINFFA_SIOLOGIC",
                "JCLKA_SIOLOGIC",
                "JLSRA_SIOLOGIC",
                "JCEA_SIOLOGIC",
                "JTSDATA0A_SIOLOGIC",
                "JTXDATA1A_SIOLOGIC",
                "JTXDATA0A_SIOLOGIC",
                "JDIRECTIONA_SIOLOGIC",
                "JMOVEA_SIOLOGIC",
                "JLOADNA_SIOLOGIC",
                "DIA_SIOLOGIC",
                "INDDA_SIOLOGIC",
                "IOLTOA_SIOLOGIC",
                "IOLDODA_SIOLOGIC",
                "IOLDOA_SIOLOGIC",
                "IOLDOIA_SIOLOGIC",
                "PADDIA_SIOLOGIC",
                "JDIA",
                "IOLDOA",
                "PADDTA_PIO",
                "PADDOA_PIO",
                "IOLTOA_PIO",
                "IOLDOA_PIO",
                "JPADDIA_PIO",
                "LVDSA_PIO",
                "INRDA_PIO",
                "JCFLAGB_SIOLOGIC",
                "JRXDATA1B_SIOLOGIC",
                "JRXDATA0B_SIOLOGIC",
                "JINFFB_SIOLOGIC",
                "JCLKB_SIOLOGIC",
                "JLSRB_SIOLOGIC",
                "JCEB_SIOLOGIC",
                "JTSDATA0B_SIOLOGIC",
                "JTXDATA1B_SIOLOGIC",
                "JTXDATA0B_SIOLOGIC",
                "JDIRECTIONB_SIOLOGIC",
                "JMOVEB_SIOLOGIC",
                "JLOADNB_SIOLOGIC",
                "DIB_SIOLOGIC",
                "INDDB_SIOLOGIC",
                "IOLTOB_SIOLOGIC",
                "IOLDODB_SIOLOGIC",
                "IOLDOB_SIOLOGIC",
                "IOLDOIB_SIOLOGIC",
                "PADDIB_SIOLOGIC",
                "JDIB",
                "IOLDOB",
                "PADDTB_PIO",
                "PADDOB_PIO",
                "IOLTOB_PIO",
                "IOLDOB_PIO",
                "JPADDIB_PIO",
                "LVDSB_PIO",
                "INRDB_PIO",
            ] {
                let wire = self.rc_io_wire(cell, wn);
                self.claim_node(wire);
            }
            for (wt, wf) in [
                ("DIA_SIOLOGIC", "JDIA"),
                ("IOLDOIA_SIOLOGIC", "IOLDOA_SIOLOGIC"),
                ("PADDIA_SIOLOGIC", "JPADDIA_PIO"),
                ("JDIA", "INDDA_SIOLOGIC"),
                ("JDIA", "JPADDIA_PIO"),
                ("IOLDOA", "IOLDODA_SIOLOGIC"),
                ("IOLDOA", "IOLDOA_SIOLOGIC"),
                ("IOLTOA_PIO", "IOLTOA_SIOLOGIC"),
                ("IOLDOA_PIO", "IOLDOA"),
                ("DIB_SIOLOGIC", "JDIB"),
                ("IOLDOIB_SIOLOGIC", "IOLDOB_SIOLOGIC"),
                ("PADDIB_SIOLOGIC", "JPADDIB_PIO"),
                ("JDIB", "INDDB_SIOLOGIC"),
                ("JDIB", "JPADDIB_PIO"),
                ("IOLDOB", "IOLDODB_SIOLOGIC"),
                ("IOLDOB", "IOLDOB_SIOLOGIC"),
                ("IOLTOB_PIO", "IOLTOB_SIOLOGIC"),
                ("IOLDOB_PIO", "IOLDOB"),
            ] {
                let wt = self.rc_io_wire(cell, wt);
                let wf = self.rc_io_wire(cell, wf);
                self.claim_pip(wt, wf);
            }
            for name in ["PT31A", "IOL_T31A", "PT31B", "IOL_T31B"] {
                self.dummy_sites.insert(name.to_string());
            }
        }
    }

    pub(super) fn process_clkdiv_ecp5(&mut self) {
        for (lr, edge) in [('L', Dir::W), ('R', Dir::E)] {
            let cell = self.chip.clk_emid(edge);
            let ci_base = 6;
            {
                let mut ex = self.extract_switchbox(cell.bel(bslots::CLK_INT));
                ex.output_bind_rc_io("CLKI0", wires::IMUX_CLKDIV_CLKI[0].cell(ci_base))
                    .output_bind_rc_io("CLKI1", wires::IMUX_CLKDIV_CLKI[1].cell(ci_base));
                for i in 0..2 {
                    for bank_idx in 0..2 {
                        let bcrd_eclk = cell.bel(bslots::ECLKSYNC[bank_idx * 2 + i]);
                        let wire_eclk = ex.ctx.naming.bel_wire(bcrd_eclk, "ECLK");
                        ex.input_xlat(wire_eclk);
                    }
                }
            }

            for i in 0..2 {
                let bcrd = cell.bel(bslots::CLKDIV[i]);
                self.hybrid_out_rc_io(
                    bcrd,
                    format!("JCDIVX_CLKDIV{i}"),
                    wires::OUT_CLKDIV_CDIVX[i].cell(ci_base),
                );

                self.extract_bel(bcrd)
                    .name(format!("CLKDIV_{lr}{i}"))
                    .input_xlat_rc_io(bcls::CLKDIV::CLKI, format!("CLKI_CLKDIV{i}"))
                    .input_xlat_rc_io(bcls::CLKDIV::RST, format!("JRST_CLKDIV{i}"))
                    .input_xlat_rc_io(bcls::CLKDIV::ALIGNWD, format!("JALIGNWD_CLKDIV{i}"))
                    .output_xlat_rc_io(bcls::CLKDIV::CDIVX, format!("JCDIVX_CLKDIV{i}"));
            }
        }
    }

    pub(super) fn process_pcsclkdiv_ecp5(&mut self) {
        let cell_tile = self.chip.clk_emid(Dir::S);
        let cell = cell_tile.delta(-1, 0);
        let bcrd_int = cell_tile.bel(bslots::CLK_INT);
        let serdes_cols = Vec::from_iter(
            self.chip
                .columns
                .ids()
                .filter(|&col| self.chip.columns[col].io[DirV::S] == IoGroupKind::Serdes),
        );

        {
            let mut ex = self.extract_switchbox(bcrd_int);
            ex.cell(cell);
            for i in 0..2 {
                if ex.ctx.skip_serdes || i >= serdes_cols.len() {
                    ex.edge(
                        wires::IMUX_CLKDIV_CLKI[i].cell(8),
                        [wires::IMUX_CLK[0].cell(7), wires::IMUX_CLK[1].cell(8)][i],
                    );
                    for j in 0..2 {
                        ex.edge(
                            wires::IMUX_CLKDIV_CLKI[i].cell(8),
                            wires::OUT_SERDES_FF_RX_PCLK[j].cell(20 + i),
                        )
                        .edge(
                            wires::IMUX_CLKDIV_CLKI[i].cell(8),
                            wires::OUT_SERDES_FF_TX_PCLK[j].cell(20 + i),
                        );
                    }
                } else {
                    ex.output_bind_rc(format!("PCSCDIVI{i}"), wires::IMUX_CLKDIV_CLKI[i].cell(8))
                        .input_xlat_rc(format!("JPCSCDIVCIB{i}"));
                    let abcd = ['A', 'B'][i];
                    for j in 0..2 {
                        ex.input_xlat_rc_nudge_rebind(
                            format!("JPCS{abcd}TXCLK{j}"),
                            defs::ecp5::tcls::CLK_S::SERDES[i],
                        )
                        .input_xlat_rc_nudge_rebind(
                            format!("JPCS{abcd}RXCLK{j}"),
                            defs::ecp5::tcls::CLK_S::SERDES[i],
                        );
                    }
                }
            }
        }
        if self.skip_serdes {
            return;
        }

        for i in 0..2 {
            let bcrd = cell_tile.bel(bslots::PCSCLKDIV[i]);
            let mut bel = self.extract_bel_cell(bcrd, cell);
            bel.name(format!("PCSCLKDIV{i}"))
                .output_bind_rc(
                    bcls::PCSCLKDIV::CDIV1,
                    format!("CDIV1_PCSCLKDIV{i}"),
                    wires::OUT_CLKDIV_CDIV1[i].cell(8),
                )
                .output_bind_rc(
                    bcls::PCSCLKDIV::CDIVX,
                    format!("CDIVX_PCSCLKDIV{i}"),
                    wires::OUT_CLKDIV_CDIVX[i].cell(8),
                );

            if i < serdes_cols.len() {
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
