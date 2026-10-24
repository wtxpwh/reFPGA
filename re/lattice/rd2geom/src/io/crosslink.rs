use prjcombine_ecp::{
    chip::{IoKind, SpecialLocKey},
    defs::{bcls, bslots, tcls, wires},
};
use prjcombine_entity::EntityId;
use prjcombine_interconnect::{
    db::{CellSlotIdExt, WireSlotIdExt},
    dir::Dir,
    grid::{BelCoord, CellCoord, ColId},
};

use crate::{ChipContext, RcMode};

impl ChipContext<'_> {
    pub(super) fn prep_io_crosslink(&mut self) {
        for (tcid, num_io) in [(tcls::IO_S4, 4), (tcls::IO_S1A, 1), (tcls::IO_S1B, 1)] {
            for &tcrd in &self.edev.tile_index[tcid] {
                for i in 0..num_io {
                    let bcrd = tcrd.bel(bslots::IO[i]);
                    let (cell_io, abcd) = self.xlat_io_loc_crosslink(bcrd);
                    let paddi = self.rc_io_wire(cell_io, &format!("JPADDI{abcd}_PIO"));
                    self.bind_wire(paddi, bcrd.wire(wires::OUT_IO_INCK[i]));
                }
            }
        }
    }

    pub(super) fn process_bc_crosslink(&mut self) {
        for (&key, &cell_loc) in &self.chip.special_loc {
            let SpecialLocKey::Bc(bank) = key else {
                continue;
            };
            let (cell, is_corner) = if bank == 2 {
                (cell_loc.with_col(self.chip.col_w()), true)
            } else {
                (cell_loc, false)
            };
            for (bel, name, pin_out, pin_in, pname_in, name_fake) in [
                (
                    bslots::BCINRD,
                    "BCINRD",
                    "INRDENO",
                    bcls::BCINRD::INRDENI,
                    "INRDENI",
                    "INRD",
                ),
                (
                    bslots::BCLVDSO,
                    "BCLVDSO",
                    "LVDSENO",
                    bcls::BCLVDSO::LVDSENI,
                    "LVDSENI",
                    "LVDS",
                ),
            ] {
                let bcrd = cell_loc.bel(bel);

                let wire_in = if is_corner {
                    self.rc_corner_wire(cell, &format!("J{pname_in}_{name}"))
                } else {
                    self.rc_io_wire(cell, &format!("J{pname_in}_{name}"))
                };
                let wire_out = if is_corner {
                    self.rc_corner_wire(cell, &format!("{pin_out}_{name}"))
                } else {
                    self.rc_io_wire(cell, &format!("{pin_out}_{name}"))
                };
                self.claim_pip(wire_out, wire_in);
                let wire_out_out = self.claim_single_out(wire_out);

                let mut bel = self.extract_bel_cell(bcrd, cell);
                bel.name(format!("{name}{bank}"))
                    .input_xlat(pin_in, wire_in)
                    .add_wire(pin_out, wire_out)
                    .add_wire(format!("{pin_out}_OUT"), wire_out_out);

                if bank == 2 {
                    // uh. what. what the fuck.
                    let wire_fake = bel
                        .ctx
                        .rc_io_wire(cell.with_col(ColId::from_idx(13)), name_fake);
                    bel.add_wire(format!("{pin_out}_FAKE"), wire_fake);
                }
            }
        }
    }

    pub(super) fn process_eclk_crosslink(&mut self) {
        let cell_tile = self.chip.clk_emid(Dir::S);
        let cell = cell_tile.delta(-1, 0);
        {
            let mut ex = self.extract_switchbox(cell_tile.bel(bslots::CLK_INT));
            ex.cell(cell)
                .output_bind_rc_io("LECLKI0", wires::IMUX_ECLKSYNC_ECLKI[0].cell(2))
                .output_bind_rc_io("LECLKI1", wires::IMUX_ECLKSYNC_ECLKI[1].cell(2))
                .output_bind_rc_io("RECLKI0", wires::IMUX_ECLKSYNC_ECLKI[2].cell(2))
                .output_bind_rc_io("RECLKI1", wires::IMUX_ECLKSYNC_ECLKI[3].cell(2))
                .output_bind_rc_io("JECLKFB", wires::ECLKFB.cell(2))
                .input_xlat_rc_io("JLECLKCIB")
                .input_xlat_rc_io("JRECLKCIB")
                .input_xlat_rc_io("JCLKOP")
                .input_xlat_rc_io("JCLKOS")
                .input_xlat_rc_io("JPCLK10")
                .input_xlat_rc_io("JPCLK11")
                .input_xlat_rc_io("JPCLK20")
                .input_xlat_rc_io("JPCLK21")
                .input_bind_rc_io("LSYNCECLK0", wires::OUT_ECLKSYNC_ECLKO[0].cell(2))
                .input_bind_rc_io("LSYNCECLK1", wires::OUT_ECLKSYNC_ECLKO[1].cell(2))
                .input_bind_rc_io("RSYNCECLK0", wires::OUT_ECLKSYNC_ECLKO[2].cell(2))
                .input_bind_rc_io("RSYNCECLK1", wires::OUT_ECLKSYNC_ECLKO[3].cell(2));
            for (name, wire) in [
                ("JBANK2ECLK0", wires::ECLK[0].cell(1)),
                ("JBANK2ECLK1", wires::ECLK[1].cell(1)),
                ("JBANK1ECLK0", wires::ECLK[0].cell(2)),
                ("JBANK1ECLK1", wires::ECLK[1].cell(2)),
            ] {
                let wname = ex.ctx.rc_io_wire(cell, name);
                let outs = ex.ctx.xlat_output(ex.bcrd, wname);
                assert_eq!(outs.len(), 1);
                let out = outs.into_iter().next().unwrap();
                ex.permabuf(out, wire).output_bind(wname, wire);
            }
        }
        for (bank, bank_idx, lr) in [(2, 0, 'L'), (1, 1, 'R')] {
            for eclk_idx in 0..2 {
                let bcrd = cell_tile.bel(bslots::ECLKSYNC[bank_idx * 2 + eclk_idx]);

                let eclk_mux = self.rc_io_wire(cell, &format!("JBANK{bank}ECLK{eclk_idx}"));
                let eclk = self.pips_fwd[&eclk_mux]
                    .iter()
                    .copied()
                    .find(|wn| self.naming.strings[wn.suffix].starts_with("BANK"))
                    .unwrap();
                self.claim_pip(eclk, eclk_mux);
                self.bind_wire(eclk, self.io_int_wires[&eclk_mux][0]);

                self.extract_bel_cell(bcrd, cell)
                    .name(format!("ECLKSYNC{eclk_idx}_BK{bank}"))
                    .input_xlat_rc_io(
                        bcls::ECLKSYNC::STOP,
                        format!("JSTOP_ECLKSYNC{lr}{eclk_idx}"),
                    )
                    .input_xlat_rc_io(
                        bcls::ECLKSYNC::ECLKI,
                        format!("ECLKI_ECLKSYNC{lr}{eclk_idx}"),
                    )
                    .output_xlat_rc_io(
                        bcls::ECLKSYNC::ECLKO,
                        format!("ECLKO_ECLKSYNC{lr}{eclk_idx}"),
                    )
                    .add_wire("ECLK", eclk)
                    .claim_pip("ECLKO", "ECLKI");
            }
        }

        {
            let bcrd = cell_tile.bel(bslots::CLKTEST_ECLK);
            let mut bel = self.extract_bel_cell(bcrd, cell);
            bel.name("CLKTEST_ECLK");

            for i in 0..6 {
                bel.input_xlat_rc_io(bcls::CLKTEST::TESTIN[i], format!("JTESTIN{i}_CLKTEST"));
            }
        }
    }

    pub(super) fn process_ddrdll_crosslink(&mut self) {
        for (&key, &cell) in &self.chip.special_loc {
            let SpecialLocKey::Bc(bank) = key else {
                continue;
            };

            {
                let clk = self.rc_io_wire(cell, "CLK_DDRDLL");
                let clk_in = self.find_single_in(clk);
                let mut ex = self.extract_switchbox(cell.bel(bslots::BEL_INT));
                ex.output_bind(clk_in, wires::IMUX_DDRDLL_CLK.cell(0))
                    .input_xlat_rc_io("JCIBCLK");
                for i in 0..2 {
                    let bcrd_eclk = ex.ctx.chip.bel_eclksync_bank(bank, i);
                    let wire_eclk = ex.ctx.naming.bel_wire(bcrd_eclk, "ECLK");
                    ex.input_xlat(wire_eclk);
                }
            }

            self.extract_bel(cell.bel(bslots::DDRDLL))
                .name(format!("DDRDLL{bank}"))
                .suffix("DDRDLL")
                .input_xlat_rc_io(bcls::DDRDLL::CLK, "CLK_DDRDLL")
                .output_bind_step_rc_mode(
                    bcls::DDRDLL::DDRDEL,
                    "DDRDEL_DDRDLL",
                    RcMode::Io,
                    wires::OUT_DDRDLL_DDRDEL.cell(0),
                );
        }
    }

    pub(super) fn process_dlldel_crosslink(&mut self) {
        let cell_tile = self.chip.clk_emid(Dir::S);
        for (name, idx) in [("20", 0), ("21", 1), ("10", 2), ("11", 3)] {
            let cell = cell_tile.delta(-2 + (idx as i32), 0);
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
                for bank in [1, 2] {
                    let bcrd_ddrdll =
                        ex.ctx.chip.special_loc[&SpecialLocKey::Bc(bank)].bel(bslots::DDRDLL);
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

    pub fn xlat_io_loc_crosslink(&self, bcrd: BelCoord) -> (CellCoord, &'static str) {
        if self.chip.get_io_kind(bcrd) == IoKind::Io {
            let idx = bslots::IO.index_of(bcrd.slot).unwrap();
            (bcrd.cell, ["A", "B", "C", "D"][idx])
        } else {
            (bcrd.cell, "")
        }
    }

    fn process_single_io_crosslink(&mut self, bcrd: BelCoord) {
        let bank = self.chip.get_io_bank(bcrd.cell);
        let idx = bslots::IO.index_of(bcrd.slot).unwrap();
        let (cell, abcd) = self.xlat_io_loc_crosslink(bcrd);
        let (_r, c) = self.rc(cell);
        let kind = self.chip.get_io_kind(bcrd);
        let iol = match kind {
            IoKind::Io => "IOLOGIC",
            IoKind::Sio => "SIOLOGIC",
            _ => unreachable!(),
        };

        let mut bel = self.extract_bel_cell(bcrd, cell);
        bel.names([format!("PB{c}{abcd}"), format!("IOL_B{c}{abcd}")])
            .input_xlat_rc_io(bcls::IO::CLK, format!("JCLK{abcd}_{iol}"))
            .input_xlat_rc_io(bcls::IO::LSR, format!("JLSR{abcd}_{iol}"))
            .input_xlat_rc_io(bcls::IO::CE, format!("JCE{abcd}_{iol}"))
            .input_xlat_rc_io(bcls::IO::TSDATA[0], format!("JTSDATA{abcd}_{iol}"))
            .input_xlat_rc_io(bcls::IO::TXDATA[0], format!("JTXDATA0{abcd}_{iol}"))
            .output_xlat_rc_io(bcls::IO::INFF, format!("JINFF{abcd}_{iol}"));

        if kind == IoKind::Io {
            let num = if matches!(idx, 0 | 2) { 16 } else { 8 };
            for i in 1..num {
                bel.input_xlat_rc_io(bcls::IO::TXDATA[i], format!("JTXDATA{i}{abcd}_{iol}"));
            }
            for i in 0..num {
                bel.output_xlat_rc_io(bcls::IO::RXDATA[i], format!("JRXDATA{i}{abcd}_{iol}"));
            }
            bel.input_xlat_rc_io(bcls::IO::DIRECTION, format!("JDIRECTION{abcd}_{iol}"))
                .input_xlat_rc_io(bcls::IO::MOVE, format!("JMOVE{abcd}_{iol}"))
                .input_xlat_rc_io(bcls::IO::LOADN, format!("JLOADN{abcd}_{iol}"))
                .input_xlat_rc_io(bcls::IO::SLIP, format!("JSLIP{abcd}_{iol}"))
                .output_xlat_rc_io(bcls::IO::CFLAG, format!("JCFLAG{abcd}_{iol}"));
            if matches!(idx, 0 | 2) {
                bel.input_xlat_rc_io(bcls::IO::HSSEL, format!("JHSSEL{abcd}_PIO"));
            } else {
                for i in 8..16 {
                    bel.add_wire_rc_io(format!("RXDATA{i}"), format!("RXDATA{i}{abcd}_{iol}"))
                        .add_wire_rc_io(format!("TXDATA{i}"), format!("TXDATA{i}{abcd}_{iol}"));
                }
            }
        }

        if !(kind == IoKind::Io && matches!(idx, 0 | 2)) {
            bel.add_wire_rc_io("HSSEL", format!("HSSEL{abcd}_PIO"));
        }

        bel.add_wire_rc_io("PADDI_PIO", format!("JPADDI{abcd}_PIO"))
            .output_xlat_rc_io(bcls::IO::DI, format!("JDI{abcd}"))
            .add_wire_rc_io("DI_IOLOGIC", format!("DI{abcd}_{iol}"))
            .add_wire_rc_io("PADDILP_PIO", format!("PADDILP{abcd}_PIO"));
        if kind == IoKind::Io {
            bel.add_wire_rc_io("PADDI_IOLOGIC", format!("PADDI{abcd}_{iol}"))
                .claim_pip("PADDI_IOLOGIC", "PADDI_PIO")
                .add_wire_rc_io("PADDI", format!("PADDI{abcd}"))
                .claim_pip("PADDI", "PADDI_PIO")
                .add_wire_rc_io("PADDILP", format!("PADDILP{abcd}"))
                .claim_pip("PADDILP", "PADDILP_PIO")
                .add_wire_rc_io("INDD_IOLOGIC", format!("INDD{abcd}_{iol}"))
                .add_wire_rc_io("PADDIDEL", format!("PADDIDEL{abcd}"))
                .claim_pip("PADDIDEL", "INDD_IOLOGIC")
                .claim_pip("DI", "PADDI")
                .claim_pip("DI", "PADDILP")
                .claim_pip("DI", "PADDIDEL")
                .add_wire_rc_io("DI_NP", format!("DI{abcd}"))
                .claim_pip("DI_NP", "PADDI")
                .claim_pip("DI_NP", "PADDIDEL")
                .claim_pip("DI_IOLOGIC", "DI_NP");
        } else {
            bel.claim_pip("DI", "PADDI_PIO")
                .claim_pip("DI_IOLOGIC", "PADDI_PIO");
        }

        bel.add_wire_rc_io("PADDO_PIO", format!("PADDO{abcd}_PIO"))
            .add_wire_rc_io("PADDT_PIO", format!("PADDT{abcd}_PIO"))
            .input_xlat_rename_rc_io(bcls::IO::TXDATA[0], "PADDO", format!("JPADDO{abcd}"))
            .input_xlat_rename_rc_io(bcls::IO::TSDATA[0], "PADDT", format!("JPADDT{abcd}"))
            .claim_pip("PADDO_PIO", "PADDO")
            .claim_pip("PADDT_PIO", "PADDT");

        bel.add_wire_rc_io("IOLDO_IOLOGIC", format!("IOLDO{abcd}_{iol}"))
            .add_wire_rc_io("IOLTO_IOLOGIC", format!("IOLTO{abcd}_{iol}"))
            .add_wire_rc_io("IOLDO_PIO", format!("IOLDO{abcd}_PIO"))
            .add_wire_rc_io("IOLTO_PIO", format!("IOLTO{abcd}_PIO"))
            .claim_pip("IOLTO_PIO", "IOLTO_IOLOGIC");
        if kind == IoKind::Io {
            bel.add_wire_rc_io("IOLDOI_IOLOGIC", format!("IOLDOI{abcd}_{iol}"))
                .add_wire_rc_io("IOLDOD_IOLOGIC", format!("IOLDOD{abcd}_{iol}"))
                .add_wire_rc_io("IOLDO", format!("IOLDO{abcd}"))
                .claim_pip("IOLDOI_IOLOGIC", "IOLDO_IOLOGIC")
                .claim_pip("IOLDO", "IOLDO_IOLOGIC")
                .claim_pip("IOLDO", "IOLDOD_IOLOGIC")
                .claim_pip("IOLDO_PIO", "IOLDO");
        } else {
            bel.claim_pip("IOLDO_PIO", "IOLDO_IOLOGIC");
        }

        if kind == IoKind::Io {
            bel.input_xlat_rc_io(bcls::IO::ECLK, format!("ECLK{abcd}_IOLOGIC"));
        }

        for (pin, bslot_rc, pin_src, pin_fake) in [
            ("INRD", bslots::BCINRD, "INRDENO_OUT", "INRDENO_FAKE"),
            ("LVDS", bslots::BCLVDSO, "LVDSENO_OUT", "LVDSENO_FAKE"),
        ] {
            let wire = bel.add_wire_rc_io_get(pin, format!("{pin}{abcd}_PIO"));
            if kind == IoKind::Io {
                let cell_rc = bel.ctx.chip.special_loc[&SpecialLocKey::Bc(bank)];
                let bcrd_rc = cell_rc.bel(bslot_rc);
                let wire_src = bel.ctx.naming.bel_wire(
                    bcrd_rc,
                    if bank == 2 && cell.col.to_idx() >= 11 {
                        pin_fake
                    } else {
                        pin_src
                    },
                );
                bel.ctx.claim_pip(wire, wire_src);
            }
        }
    }

    pub(super) fn process_io_crosslink(&mut self) {
        for (tcid, num_io) in [(tcls::IO_S4, 4), (tcls::IO_S1A, 1), (tcls::IO_S1B, 1)] {
            for &tcrd in &self.edev.tile_index[tcid] {
                if tcid == tcls::IO_S4 {
                    let bank = self.chip.get_io_bank(tcrd.cell);
                    let mut ex = self.extract_switchbox(tcrd.bel(bslots::IO_INT));
                    ex.output_bind_rc_io("ECLKA", wires::IMUX_IO_ECLK[0].cell(0))
                        .output_bind_rc_io("ECLKB", wires::IMUX_IO_ECLK[1].cell(0))
                        .output_bind_rc_io("ECLKC", wires::IMUX_IO_ECLK[2].cell(0))
                        .output_bind_rc_io("ECLKD", wires::IMUX_IO_ECLK[3].cell(0));
                    for i in 0..2 {
                        let bcrd_eclk = ex.ctx.chip.bel_eclksync_bank(bank, i);
                        let wire_eclk = ex.ctx.naming.bel_wire(bcrd_eclk, "ECLK");
                        ex.input_xlat(wire_eclk);
                    }
                }
                for i in 0..num_io {
                    let bcrd = tcrd.bel(bslots::IO[i]);
                    self.process_single_io_crosslink(bcrd);
                }
            }
        }
    }

    pub(super) fn process_clkdiv_crosslink(&mut self) {
        let cell_tile = self.chip.clk_emid(Dir::S);
        let cell = cell_tile.delta(-1, 0);
        for i in 0..4 {
            let bcrd = cell_tile.bel(bslots::CLKDIV[i]);

            self.hybrid_out_rc_io_cell(
                bcrd,
                cell,
                format!("JCDIVX_CLKDIV{i}"),
                wires::OUT_CLKDIV_CDIVX[i].cell(2),
            );

            self.extract_bel_cell(bcrd, cell)
                .name(format!("CLKDIV{i}"))
                .input_xlat_rc_io(bcls::CLKDIV::CLKI, format!("CLKI_CLKDIV{i}"))
                .input_xlat_rc_io(bcls::CLKDIV::RST, format!("JRST_CLKDIV{i}"))
                .input_xlat_rc_io(bcls::CLKDIV::ALIGNWD, format!("JALIGNWD_CLKDIV{i}"))
                .output_xlat_rc_io(bcls::CLKDIV::CDIVX, format!("JCDIVX_CLKDIV{i}"));
        }
    }
}
