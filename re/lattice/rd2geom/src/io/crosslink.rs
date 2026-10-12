use prjcombine_ecp::{
    chip::{Chip, IoKind, SpecialIoKey, SpecialLocKey},
    defs::{bcls, bslots, tcls, wires},
};
use prjcombine_entity::EntityId;
use prjcombine_interconnect::{
    db::{LegacyBel, WireSlotIdExt},
    dir::{Dir, DirHV},
    grid::{BelCoord, CellCoord, ColId, DieIdExt},
};

use crate::ChipContext;

impl ChipContext<'_> {
    pub(super) fn prep_io_crosslink(&mut self) {
        for (tcid, num_io) in [(tcls::IO_S4, 4), (tcls::IO_S1A, 1), (tcls::IO_S1B, 1)] {
            for &tcrd in &self.edev.tile_index[tcid] {
                for i in 0..num_io {
                    let bcrd = tcrd.bel(bslots::IO[i]);
                    let (cell_io, abcd) = self.xlat_io_loc_crosslink(bcrd);
                    let paddi = self.rc_io_wire(cell_io, &format!("JPADDI{abcd}_PIO"));
                    self.bind_wire(paddi, bcrd.wire(wires::OUT_IO_PADDI[i]));
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
        let cell_tile = Chip::DIE.cell(self.chip.col_clk, self.chip.row_s());
        let cell = cell_tile.delta(-1, 0);
        for (bank, bank_idx, lr, olr) in [(2, 0, 'L', 'R'), (1, 1, 'R', 'L')] {
            let mut eclki_bpin = None;
            for eclk_idx in 0..2 {
                let wire_eclk = cell
                    .delta(i32::try_from(bank_idx).unwrap(), 0)
                    .wire(wires::ECLK[eclk_idx]);
                let bcrd = cell_tile.bel(bslots::ECLKSYNC[bank_idx * 2 + eclk_idx]);
                self.name_bel(bcrd, [format!("ECLKSYNC{eclk_idx}_BK{bank}")]);
                let mut bel = LegacyBel::default();

                let stop = self.rc_io_wire(cell, &format!("JSTOP_ECLKSYNC{lr}{eclk_idx}"));
                self.add_bel_wire(bcrd, "STOP", stop);
                bel.pins
                    .insert("STOP".into(), self.xlat_int_wire(bcrd, stop));

                let eclki = self.rc_io_wire(cell, &format!("ECLKI_ECLKSYNC{lr}{eclk_idx}"));
                self.add_bel_wire(bcrd, "ECLKI", eclki);
                let eclki_in = self.claim_single_in(eclki);
                self.add_bel_wire(bcrd, "ECLKI_IN", eclki_in);

                let eclki_int = self.rc_io_wire(cell, &format!("J{lr}ECLKCIB"));
                self.add_bel_wire_no_claim(bcrd, "ECLKI_INT", eclki_int);
                self.claim_pip(eclki_in, eclki_int);
                if eclk_idx == 0 {
                    self.claim_node(eclki_int);
                    let bpin = self.xlat_int_wire(bcrd, eclki_int);
                    eclki_bpin = Some(bpin.clone());
                    bel.pins.insert("ECLKI".into(), bpin);
                } else {
                    bel.pins.insert("ECLKI".into(), eclki_bpin.take().unwrap());
                }

                let wire_eclkfb = self.rc_io_wire(cell, "JECLKFB");
                self.add_bel_wire_no_claim(bcrd, "ECLKFB", wire_eclkfb);
                if bank_idx == 0 && eclk_idx == 0 {
                    self.claim_node(wire_eclkfb);
                }

                for pin in ["CLKOP", "CLKOS"] {
                    let eclki_pll = self.rc_io_wire(cell, &format!("J{pin}"));
                    self.add_bel_wire_no_claim(bcrd, format!("PLL_IN_{pin}"), eclki_pll);
                    self.claim_pip(eclki_in, eclki_pll);
                    if bank_idx == 0 && eclk_idx == 0 {
                        self.claim_node(eclki_pll);
                        let cell_pll = self.chip.bel_pll(DirHV::SE, 0).cell;
                        let wire_pll = self.rc_wire(cell_pll, &format!("J{pin}_PLL"));
                        self.claim_pip(eclki_pll, wire_pll);
                        self.claim_pip(wire_eclkfb, eclki_pll);
                    }
                }

                for i in 0..4 {
                    let name = ["20", "21", "10", "11"][i];
                    let pclk = self.rc_io_wire(cell, &format!("JPCLK{name}"));
                    self.add_bel_wire_no_claim(bcrd, format!("IO_IN_S{i}"), pclk);
                    self.claim_pip(eclki_in, pclk);
                    if bank_idx == 0 && eclk_idx == 0 {
                        self.claim_node(pclk);
                        let cell_dlldel = bcrd.cell.delta(-2 + (i as i32), 0);
                        let wire_dlldel = self.rc_io_wire(cell_dlldel, "JINCK");
                        self.claim_pip(pclk, wire_dlldel);
                    }
                }

                let eclko = self.rc_io_wire(cell, &format!("ECLKO_ECLKSYNC{lr}{eclk_idx}"));
                self.add_bel_wire(bcrd, "ELCKO", eclko);
                self.claim_pip(eclko, eclki);

                let eclko_out = self.rc_io_wire(cell, &format!("{lr}SYNCECLK{eclk_idx}"));
                self.add_bel_wire(bcrd, "ELCKO_OUT", eclko_out);
                self.claim_pip(eclko_out, eclko);

                let eclk_mux = self.rc_io_wire(cell, &format!("JBANK{bank}ECLK{eclk_idx}"));
                self.add_bel_wire(bcrd, "ECLK_MUX", eclk_mux);
                self.claim_pip(eclk_mux, eclko_out);
                let eclko_other = self.rc_io_wire(cell, &format!("{olr}SYNCECLK{eclk_idx}"));
                self.claim_pip(eclk_mux, eclko_other);
                bel.pins
                    .insert("ECLK".into(), self.xlat_int_wire(bcrd, eclk_mux));

                let eclk = self.pips_fwd[&eclk_mux]
                    .iter()
                    .copied()
                    .find(|wn| self.naming.strings[wn.suffix].starts_with("BANK"))
                    .unwrap();
                self.add_bel_wire(bcrd, "ECLK", eclk);
                self.bind_wire(eclk, wire_eclk);
                self.claim_pip(eclk, eclk_mux);

                self.insert_legacy_bel(bcrd, bel);
            }
        }

        {
            let bcrd = cell_tile.bel(bslots::CLKTEST_ECLK);
            self.name_bel(bcrd, ["CLKTEST_ECLK"]);
            let mut bel = LegacyBel::default();

            for i in 0..6 {
                let wire = self.rc_io_wire(cell, &format!("JTESTIN{i}_CLKTEST"));
                self.add_bel_wire(bcrd, format!("TESTIN{i}"), wire);
                bel.pins
                    .insert(format!("TESTIN{i}"), self.xlat_int_wire(bcrd, wire));
            }

            self.insert_legacy_bel(bcrd, bel);
        }
    }

    pub(super) fn process_ddrdll_crosslink(&mut self) {
        for (&key, &cell) in &self.chip.special_loc {
            let SpecialLocKey::Bc(bank) = key else {
                continue;
            };
            let bcrd = cell.bel(bslots::DDRDLL);
            self.name_bel(bcrd, [format!("DDRDLL{bank}")]);
            let mut bel = self.extract_simple_legacy_bel(bcrd, cell, "DDRDLL");

            let clk = self.rc_io_wire(cell, "CLK_DDRDLL");
            self.add_bel_wire(bcrd, "CLK", clk);
            let clk_in = self.claim_single_in(clk);
            self.add_bel_wire(bcrd, "CLK_IN", clk_in);

            let clk_int = self.rc_io_wire(cell, "JCIBCLK");
            self.add_bel_wire(bcrd, "CLK_INT", clk_int);
            self.claim_pip(clk_in, clk_int);
            bel.pins
                .insert("CLK".into(), self.xlat_int_wire(bcrd, clk_int));

            for i in 0..2 {
                let bcrd_eclk = self.chip.bel_eclksync_bank(bank, i);
                let wire_eclk = self.naming.bel_wire(bcrd_eclk, "ECLK");
                self.claim_pip(clk_in, wire_eclk);
            }

            let ddrdel = self.rc_io_wire(cell, "DDRDEL_DDRDLL");
            self.add_bel_wire(bcrd, "DDRDEL", ddrdel);
            let ddrdel_out = self.claim_single_out(ddrdel);
            self.add_bel_wire(bcrd, "DDRDEL_OUT", ddrdel_out);

            self.insert_legacy_bel(bcrd, bel);
        }
    }

    pub(super) fn process_dlldel_crosslink(&mut self) {
        let cell_tile = Chip::DIE.cell(self.chip.col_clk, self.chip.row_s());
        for (name, idx) in [("20", 0), ("21", 1), ("10", 2), ("11", 3)] {
            let Some(&io) = self
                .chip
                .special_io
                .get(&SpecialIoKey::Clock(Dir::S, idx as u8))
            else {
                continue;
            };
            let cell = cell_tile.delta(-2 + (idx as i32), 0);
            let bcrd = cell_tile.bel(bslots::DLLDEL[idx]);
            self.name_bel(bcrd, [format!("DLLDEL_{name}")]);

            let (cell_io, abcd) = self.xlat_io_loc_crosslink(io);
            let paddi_pio = self.rc_io_wire(cell_io, &format!("JPADDI{abcd}_PIO"));

            let paddi = self.rc_io_wire(cell, "JPADDI");
            self.add_bel_wire(bcrd, "PADDI", paddi);
            self.claim_pip(paddi, paddi_pio);

            let a = self.rc_io_wire(cell, "JA_DLLDEL");
            self.add_bel_wire(bcrd, "A", a);
            self.claim_pip(a, paddi_pio);

            let z = self.rc_io_wire(cell, "Z_DLLDEL");
            self.add_bel_wire(bcrd, "Z", z);

            let z_out = self.rc_io_wire(cell, "DLLDEL");
            self.add_bel_wire(bcrd, "Z_OUT", z_out);
            self.claim_pip(z_out, z);

            let inck = self.rc_io_wire(cell, "JINCK");
            self.add_bel_wire(bcrd, "INCK", inck);
            self.claim_pip(inck, paddi);
            self.claim_pip(inck, z_out);

            let ddrdel = self.rc_io_wire(cell, "DDRDEL_DLLDEL");
            self.add_bel_wire(bcrd, "DDRDEL", ddrdel);
            let ddrdel_in = self.rc_io_wire(cell, "DDRDEL");
            self.add_bel_wire(bcrd, "DDRDEL_IN", ddrdel_in);
            self.claim_pip(ddrdel, ddrdel_in);

            for bank in [1, 2] {
                let bcrd_ddrdll =
                    self.chip.special_loc[&SpecialLocKey::Bc(bank)].bel(bslots::DDRDLL);
                let wire_ddrdll = self.naming.bel_wire(bcrd_ddrdll, "DDRDEL_OUT");
                self.claim_pip(ddrdel_in, wire_ddrdll);
            }

            self.insert_simple_legacy_bel(bcrd, cell, "DLLDEL");
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
        let bank = self.chip.get_io_bank(bcrd);
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
            .input_xlat_rc_io(bcls::IO::TSDATA, format!("JTSDATA{abcd}_{iol}"))
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
            .input_xlat_rename_rc_io(bcls::IO::TSDATA, "PADDT", format!("JPADDT{abcd}"))
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
                    let bank = self.chip.get_io_bank(tcrd.bel(bslots::IO[0]));
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
        let cell_tile = Chip::DIE.cell(self.chip.col_clk, self.chip.row_s());
        let cell = cell_tile.delta(-1, 0);
        for i in 0..4 {
            let bcrd = cell_tile.bel(bslots::CLKDIV[i]);
            self.name_bel(bcrd, [format!("CLKDIV{i}")]);
            let mut bel = LegacyBel::default();

            for pin in ["ALIGNWD", "RST", "CDIVX"] {
                let wire = self.rc_io_wire(cell, &format!("J{pin}_CLKDIV{i}"));
                self.add_bel_wire(bcrd, pin, wire);
                bel.pins.insert(pin.into(), self.xlat_int_wire(bcrd, wire));
            }

            let clki = self.rc_io_wire(cell, &format!("CLKI_CLKDIV{i}"));
            self.add_bel_wire(bcrd, "CLKI", clki);

            let bcrd_eclk = bcrd.bel(bslots::ECLKSYNC[i]);
            let wire_eclk = self.naming.bel_wire(bcrd_eclk, "ECLK_MUX");
            self.claim_pip(clki, wire_eclk);

            self.insert_legacy_bel(bcrd, bel);
        }
    }
}
