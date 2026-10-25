use prjcombine_ecp::{
    chip::{Chip, ChipKind, IoGroupKind, MachXo2Kind, RowKind, SpecialIoKey, SpecialLocKey},
    defs::{bcls, bslots, wires},
};
use prjcombine_entity::EntityId;
use prjcombine_interconnect::{
    db::WireSlotIdExt,
    dir::{Dir, DirH, DirHV, DirV},
    grid::{BelCoord, CellCoord, ColId, DieIdExt, RowId},
};

use crate::ChipContext;

impl ChipContext<'_> {
    pub(super) fn prep_io_machxo2(&mut self) {
        for cell in self.edev.die_cells(Chip::DIE) {
            for iob in 0..4 {
                let bcrd = cell.bel(bslots::IO[iob]);
                if self.edev.has_bel(bcrd) && !self.is_machnx_io(bcrd) {
                    let abcd = ['A', 'B', 'C', 'D'][iob];
                    self.hybrid_out_rc_io(
                        bcrd,
                        format!("JDI{abcd}"),
                        wires::OUT_IO_INCK[iob].cell(0),
                    );
                }
            }
        }
    }

    fn get_nominal_bc_cell_machxo2(&self, bank: u32) -> CellCoord {
        let has_bank4 = self.chip.special_loc.contains_key(&SpecialLocKey::Bc(4));
        let (col, row) = match bank {
            0 => (self.chip.col_clk - 1, self.chip.row_n()),
            1 => (self.chip.col_e(), self.chip.row_clk),
            2 => (self.chip.col_clk - 1, self.chip.row_s()),
            3 if !has_bank4 => (self.chip.col_w(), self.chip.row_clk),
            3 if has_bank4 => (self.chip.col_w(), self.chip.row_s()),
            4 if has_bank4 => (self.chip.col_w(), self.chip.row_clk),
            5 if has_bank4 => (self.chip.col_w(), self.chip.row_n()),
            _ => unreachable!(),
        };
        Chip::DIE.cell(col, row)
    }

    pub(super) fn process_bc_machxo2(&mut self) {
        for (&key, &cell_loc) in &self.chip.special_loc {
            let SpecialLocKey::Bc(bank) = key else {
                continue;
            };
            let cell = self.get_nominal_bc_cell_machxo2(bank);
            for (bel, suffix, pin_out, pin_in) in [
                (bslots::BCPG, "BCPG", "PGENO", "PGENI"),
                (bslots::BCINRD, "BCINRD", "INRDENO", "INRDENI"),
                (bslots::BCLVDSO, "BCLVDSO", "LVDSENO", "LVDSENI"),
                (
                    bslots::BCSLEWRATE,
                    "BCSLEWRATE",
                    "SLEWRATEENO",
                    "SLEWRATEENI",
                ),
            ] {
                let bcrd = cell_loc.bel(bel);
                if self.edev.has_bel(bcrd) {
                    let wire_out = if bel == bslots::BCSLEWRATE {
                        self.rc_io_wire(cell, &format!("J{pin_out}_{suffix}"))
                    } else {
                        let wire_in = self.rc_io_wire(cell, &format!("J{pin_in}_{suffix}"));
                        let wire_out = self.rc_io_wire(cell, &format!("{pin_out}_{suffix}"));
                        self.claim_pip(wire_out, wire_in);
                        wire_out
                    };
                    let wire_out_out = self.claim_single_out(wire_out);
                    let name = if bel == bslots::BCLVDSO {
                        "BCLVDSO".to_string()
                    } else {
                        format!("{suffix}{bank}")
                    };
                    self.extract_bel_cell(bcrd, cell)
                        .name(name)
                        .suffix(suffix)
                        .add_wire(pin_out, wire_out)
                        .add_wire(format!("{pin_out}_OUT"), wire_out_out);
                }
            }
        }
    }

    pub(super) fn process_eclk_machxo2(&mut self) {
        let is_smol = self.chip.rows[self.chip.row_clk].kind != RowKind::Ebr;
        if is_smol {
            return;
        }
        for edge in [DirV::S, DirV::N] {
            let cell = self.chip.bel_eclksync(Dir::V(edge), 0).cell;

            {
                let mut ex = self.extract_switchbox(cell.bel(bslots::CLK_INT));
                ex.output_bind_rc("ECLKI0", wires::IMUX_ECLKSYNC_ECLKI[0].cell(0))
                    .output_bind_rc("ECLKI1", wires::IMUX_ECLKSYNC_ECLKI[1].cell(0))
                    .input_xlat_rc("JECLKCIB0")
                    .input_xlat_rc("JECLKCIB1")
                    .input_xlat_rc("JECLKBRG0")
                    .input_xlat_rc("JECLKBRG1")
                    .input_xlat_rc("JINECK0")
                    .input_xlat_rc("JINECK1");
                for (plli, hv) in [(0, DirHV::NW), (1, DirHV::NE)] {
                    if ex.ctx.chip.try_bel_pll(hv, 0).is_some() {
                        ex.input_xlat_rc(format!("JPLLCLKOP{plli}"))
                            .input_xlat_rc(format!("JPLLCLKOS{plli}"));
                    }
                }
            }

            let mut eclko_outs = vec![];
            for idx in 0..2 {
                let bcrd = self.chip.bel_eclksync(Dir::V(edge), idx);

                let eclko = self.rc_wire(cell, &format!("JECLKO{idx}_ECLKSYNC"));
                let eclko_out = self.pips_fwd[&eclko]
                    .iter()
                    .copied()
                    .find(|&wn| {
                        let suffix = self.naming.strings[wn.suffix].as_str();
                        suffix.starts_with("TECLK") || suffix.starts_with("BECLK")
                    })
                    .unwrap();
                self.hybrid_out(bcrd, eclko, wires::ECLK[idx].cell(0));
                self.bind_wire(eclko_out, cell.wire(wires::ECLK[idx]));
                self.claim_pip(eclko_out, eclko);

                self.extract_bel(bcrd)
                    .name(format!(
                        "{bt}ECLKSYNC{idx}",
                        bt = match edge {
                            DirV::S => 'B',
                            DirV::N => 'T',
                        }
                    ))
                    .input_xlat_rc(bcls::ECLKSYNC::STOP, format!("JSTOP{idx}_ECLKSYNC"))
                    .input_xlat_rc(bcls::ECLKSYNC::ECLKI, format!("ECLKI{idx}_ECLKSYNC"))
                    .output_xlat(bcls::ECLKSYNC::ECLKO, eclko)
                    .add_wire("ECLKO_OUT", eclko_out)
                    .claim_pip("ECLKO", "ECLKI");

                eclko_outs.push(eclko_out);
            }

            if edge == DirV::S {
                self.extract_switchbox(cell.bel(bslots::CLK_INT))
                    .output_bind_rc("JPLLCLKFB0", wires::PLLCLKFB[0].cell(0))
                    .output_bind_rc("JPLLCLKFB1", wires::PLLCLKFB[1].cell(0))
                    .input_bind_rc("CLKFBBUF0", wires::OUT_CLKFBBUF_Z[0].cell(0))
                    .input_bind_rc("CLKFBBUF1", wires::OUT_CLKFBBUF_Z[1].cell(0))
                    .input_xlat(eclko_outs[0])
                    .input_xlat(eclko_outs[1]);
                for idx in 0..2 {
                    let bcrd = cell.bel(bslots::CLKFBBUF[idx]);
                    let mut bel = self.extract_bel(bcrd);
                    bel.name(format!("CLKFBBUF{idx}"))
                        .output_xlat_rc(bcls::CLKFBBUF::Z, format!("Z{idx}_CLKFBBUF"));

                    let hv = [DirHV::NW, DirHV::NE][idx];
                    if bel.ctx.chip.try_bel_pll(hv, 0).is_some() {
                        bel.input_xlat_rc(bcls::CLKFBBUF::A, format!("JA{idx}_CLKFBBUF"));
                    } else {
                        bel.add_wire_rc("A", format!("JA{idx}_CLKFBBUF"))
                            .input(bcls::CLKFBBUF::A, wires::OUT_PLL_CLKOP[0].cell(idx + 2));
                    }
                }
            }
        }
    }

    pub(super) fn process_dqsdll_machxo2(&mut self) {
        for edge in [DirV::S, DirV::N] {
            let Some(&cell) = self
                .chip
                .special_loc
                .get(&SpecialLocKey::DqsDll(Dir::V(edge)))
            else {
                continue;
            };
            let bt = match edge {
                DirV::S => 'B',
                DirV::N => 'T',
            };

            let bcrd_eclksync = self.chip.bel_eclksync(Dir::V(edge), 0);
            let eclk0 = self.naming.bel_wire(bcrd_eclksync, "ECLKO_OUT");
            let bcrd_eclksync = self.chip.bel_eclksync(Dir::V(edge), 1);
            let eclk1 = self.naming.bel_wire(bcrd_eclksync, "ECLKO_OUT");

            self.extract_switchbox(cell.bel(bslots::BEL_INT))
                .output_bind_rc("DQSDLLCLK", wires::IMUX_DDRDLL_CLK.cell(0))
                .input_xlat_rc("JDQSDLLSCLK")
                .input_xlat(eclk0)
                .input_xlat(eclk1);

            let bcrd = cell.bel(bslots::DQSDLL);
            let mut bel = self.extract_bel(bcrd);
            bel.names([format!("{bt}DQSDLL"), format!("{bt}DQSDLLTEST")])
                .suffix("DQSDLL")
                .add_wire_rc("DQSDEL", "JDQSDEL_DQSDLL")
                .input_xlat_rc(bcls::DDRDLL::CLK, "CLK_DQSDLL")
                .output_xlat_rc(bcls::DDRDLL::TEST_DIVOSC, "JDIVOSC_DQSDLLTEST");
            for i in 0..7 {
                bel.output_xlat_rc(bcls::DDRDLL::TEST_SDOUT[i], format!("JSDOUT{i}_DQSDLLTEST"));
            }
        }
    }

    fn is_machnx_io(&self, bcrd: BelCoord) -> bool {
        if self.chip.kind != ChipKind::MachXo2(MachXo2Kind::MachNx) {
            return false;
        }
        if bcrd.col != self.chip.col_w() {
            return false;
        }
        let idx = bslots::IO.index_of(bcrd.slot).unwrap();
        matches!(
            (bcrd.row.to_idx(), idx),
            (2 | 3 | 4 | 5 | 6 | 9 | 10, _)
                | (11, 1 | 2)
                | (12, 1)
                | (13, 0 | 1)
                | (14, 0)
                | (16, 1)
        )
    }

    pub(super) fn process_dqs_machxo2(&mut self) {
        let is_smol = self.chip.rows[self.chip.row_clk].kind != RowKind::Ebr;
        if is_smol {
            return;
        }
        if self.chip.kind == ChipKind::MachXo2(MachXo2Kind::MachXo3L) {
            return;
        }
        let has_dqs = self.chip.kind == ChipKind::MachXo2(MachXo2Kind::MachXo2);
        let cell = Chip::DIE.cell(self.chip.col_e(), self.chip.row_clk);
        for i in 0..2 {
            let bcrd = cell.bel(bslots::DQS[i]);
            if !has_dqs {
                self.name_bel_null(bcrd);
                for pin in ["DDRCLKPOL", "DQSR90", "DQSW90"] {
                    let cell_io = [cell.delta(0, 2), cell.delta(0, -2)][i];
                    let wire_io = self.rc_io_wire(cell_io, &format!("{pin}A_RIOLOGIC"));
                    let wire_out = self.find_single_in(wire_io);
                    self.add_bel_wire(bcrd, format!("{pin}_OUT"), wire_out);
                }
            } else {
                let mut bel = self.extract_bel(bcrd);
                bel.name(format!("DQS{i}"))
                    .input_xlat_rc_io(bcls::DQS::SCLK, format!("JSCLK{i}_DQS"))
                    .input_xlat_rc_io(bcls::DQS::RST, format!("JRST{i}_DQS"))
                    .input_xlat_rc_io(bcls::DQS::READ[0], format!("JREAD{i}_DQS"))
                    .input_xlat_rc_io(bcls::DQS::READCLKSEL[0], format!("JREADCLKSEL0_{i}_DQS"))
                    .input_xlat_rc_io(bcls::DQS::READCLKSEL[1], format!("JREADCLKSEL1_{i}_DQS"))
                    .output_xlat_rc_io(bcls::DQS::DATAVALID, format!("JDATAVALID{i}_DQS"))
                    .output_xlat_rc_io(bcls::DQS::BURSTDET, format!("JBURSTDET{i}_DQS"));
                for (pin, pname) in [
                    (bcls::DQS::DDRCLKPOL, "DDRCLKPOL"),
                    (bcls::DQS::DQSR90, "DQSR90"),
                    (bcls::DQS::DQSW90, "DQSW90"),
                ] {
                    let cell_io = [cell.delta(0, 2), cell.delta(0, -2)][i];
                    let wire_io = bel.ctx.rc_io_wire(cell_io, &format!("{pname}A_RIOLOGIC"));
                    let wire_out = bel.ctx.find_single_in(wire_io);

                    bel.output_xlat_rename(pin, format!("{pname}_OUT"), wire_out)
                        .add_wire_rc_io(
                            pname,
                            if pname.ends_with('0') {
                                format!("J{pname}_{i}_DQS")
                            } else {
                                format!("J{pname}{i}_DQS")
                            },
                        )
                        .claim_pip(format!("{pname}_OUT"), pname);
                }

                let cell_dqsdll = bel.ctx.chip.special_loc[&SpecialLocKey::DqsDll(Dir::N)];
                let wire_dqsdll = bel.ctx.rc_wire(cell_dqsdll, "JDQSDEL_DQSDLL");
                let dqsdel = bel.add_wire_rc_io_get("DQSDEL", format!("JDQSDEL{i}_DQS"));
                bel.ctx.claim_pip(dqsdel, wire_dqsdll);

                let dqsi = bel.add_wire_rc_io_get("DQSI", format!("JDQSI{i}_DQS"));
                let io = bel.ctx.chip.special_io[&SpecialIoKey::DqsE(i as u8)];
                let io_idx = bslots::IO.index_of(io.slot).unwrap();
                let wire_io = bel.ctx.rc_io_wire(
                    io.cell,
                    &format!("JDI{abcd}", abcd = ['A', 'B', 'C', 'D'][io_idx]),
                );
                bel.ctx.claim_pip(dqsi, wire_io);
            }
        }
    }

    fn process_single_io_machxo2(&mut self, bcrd: BelCoord) {
        let is_smol = self.chip.rows[self.chip.row_clk].kind != RowKind::Ebr;
        let idx = bslots::IO.index_of(bcrd.slot).unwrap();
        let abcd = ['A', 'B', 'C', 'D'][idx];
        let cell = bcrd.cell;
        let (r, c) = self.rc(cell);
        let edge = self.chip.get_io_edge(cell);
        let mut names = match edge {
            Dir::W => [format!("PL{r}{abcd}"), format!("IOL_L{r}{abcd}")],
            Dir::E => [format!("PR{r}{abcd}"), format!("IOL_R{r}{abcd}")],
            Dir::S => [format!("PB{c}{abcd}"), format!("IOL_B{c}{abcd}")],
            Dir::N => [format!("PT{c}{abcd}"), format!("IOL_T{c}{abcd}")],
        };
        if self.chip.kind == ChipKind::MachXo2(MachXo2Kind::MachNx)
            && bcrd.col == self.chip.col_w()
            && let Some(name) = match (bcrd.row.to_idx(), idx) {
                (16, 1) => Some("NXBOOT_MCSN"),
                (11, 1) => Some("NX_PROGRAMN"),
                (11, 2) => Some("NX_JTAGEN"),
                (10, 0) => Some("ICC00"),
                (10, 1) => Some("ICC01"),
                (10, 2) => Some("ICC02"),
                (10, 3) => Some("ICC03"),
                (9, 0) => Some("ICC04"),
                (9, 1) => Some("ICC05"),
                (9, 2) => Some("ICC06"),
                (9, 3) => Some("ICC07"),
                (6, 0) => Some("ICC08"),
                (6, 1) => Some("ICC09"),
                (6, 2) => Some("ICC10"),
                (6, 3) => Some("ICC11"),
                (5, 0) => Some("ICC12"),
                (5, 1) => Some("ICC13"),
                (5, 2) => Some("ICC14"),
                (5, 3) => Some("ICC15"),
                (4, 0) => Some("ICC16"),
                (4, 1) => Some("ICC17"),
                (4, 2) => Some("ICC18"),
                (4, 3) => Some("ICC19"),
                (3, 0) => Some("ICC20"),
                (3, 1) => Some("ICC21"),
                (3, 2) => Some("ICC22"),
                (3, 3) => Some("ICC23"),
                (2, 0) => Some("ICC24"),
                (2, 1) => Some("ICC25"),
                (2, 2) => Some("ICC26"),
                (2, 3) => Some("ICC27"),
                _ => None,
            }
        {
            names[0] = name.to_string();
        }
        if self.chip.kind == ChipKind::MachXo2(MachXo2Kind::MachNx)
            && bcrd
                == Chip::DIE
                    .cell(ColId::from_idx(14), self.chip.row_n())
                    .bel(bslots::IO[2])
        {
            // what. this is actually the XO3 TDO -> XO5 TDI pad btw.
            names[0] = "Unused".to_string();
        }
        let bank = self.chip.get_io_bank(bcrd.cell);
        let iol = if is_smol {
            "IOLOGIC"
        } else {
            match (bank, idx) {
                (0, 0) => "TIOLOGIC",
                (0, 2) => "TSIOLOGIC",
                (2, 0) => "BIOLOGIC",
                (2, 2) => "BSIOLOGIC",
                (1, _) if self.chip.kind != ChipKind::MachXo2(MachXo2Kind::MachXo3L) => "RIOLOGIC",
                _ => "IOLOGIC",
            }
        };

        let mut bel = self.extract_bel(bcrd);
        bel.names(names)
            .input_xlat_rc_io(bcls::IO::CLK, format!("JCLK{abcd}_{iol}"))
            .input_xlat_rc_io(bcls::IO::LSR, format!("JLSR{abcd}_{iol}"))
            .input_xlat_rc_io(bcls::IO::CE, format!("JCE{abcd}_{iol}"))
            .input_xlat_rc_io(bcls::IO::TS, format!("JTS{abcd}_{iol}"))
            .input_xlat_rc_io(bcls::IO::OPOS[0], format!("JOPOS{abcd}_{iol}"))
            .input_xlat_rc_io(bcls::IO::ONEG[0], format!("JONEG{abcd}_{iol}"))
            .output_xlat_rc_io(bcls::IO::IP, format!("JIP{abcd}_{iol}"))
            .output_xlat_rc_io(bcls::IO::IN, format!("JIN{abcd}_{iol}"));

        match iol {
            "TIOLOGIC" => {
                for i in 0..8 {
                    bel.input_xlat_rc_io(bcls::IO::TXD[i], format!("JTXD{i}{abcd}_{iol}"));
                }
            }
            "TSIOLOGIC" => {
                for i in 0..4 {
                    bel.input_xlat_rc_io(bcls::IO::TXD[i], format!("JTXD{i}{abcd}_{iol}"));
                }
            }
            "BIOLOGIC" | "BSIOLOGIC" => {
                bel.input_xlat_rc_io(bcls::IO::SLIP, format!("JSLIP{abcd}_{iol}"));
                for i in 0..5 {
                    bel.input_xlat_rc_io(bcls::IO::DEL[i], format!("JDEL{i}{abcd}_{iol}"));
                }
                for i in 0..4 {
                    bel.output_xlat_rc_io(bcls::IO::RXD[i], format!("JRXD{i}{abcd}_{iol}"));
                }
            }
            _ => (),
        }

        if iol == "BIOLOGIC" {
            for i in 0..8 {
                bel.output_xlat_rc_io(bcls::IO::RXDA[i], format!("JRXDA{i}_{iol}"));
            }
        }

        if bel.ctx.is_machnx_io(bcrd) {
            bel.add_wire_rc_io("DI", format!("JDI{abcd}")).output_raw(
                bcls::IO::DI,
                [wires::OUT_Q[idx].cell(0), wires::OUT_IO_INCK[idx].cell(0)].into(),
            );
        } else {
            bel.output_xlat_rc_io(bcls::IO::DI, format!("JDI{abcd}"));
        }

        bel.add_wire_rc_io("PADDI_PIO", format!("JPADDI{abcd}_PIO"))
            .add_wire_rc_io("DI_IOLOGIC", format!("DI{abcd}_{iol}"))
            .add_wire_rc_io("PADDI_IOLOGIC", format!("PADDI{abcd}_{iol}"))
            .add_wire_rc_io("INDD", format!("INDD{abcd}_{iol}"))
            .claim_pip("PADDI_IOLOGIC", "PADDI_PIO")
            .claim_pip("DI_IOLOGIC", "DI")
            .claim_pip("DI", "INDD")
            .claim_pip("DI", "PADDI_PIO");

        bel.add_wire_rc_io("PADDO_PIO", format!("PADDO{abcd}_PIO"))
            .add_wire_rc_io("PADDT_PIO", format!("PADDT{abcd}_PIO"));

        if !bel.ctx.is_machnx_io(bcrd) {
            bel.input_xlat_rename_rc_io(bcls::IO::OPOS[0], "PADDO", format!("JPADDO{abcd}"))
                .input_xlat_rename_rc_io(bcls::IO::TS, "PADDT", format!("JPADDT{abcd}"))
                .claim_pip("PADDO_PIO", "PADDO")
                .claim_pip("PADDT_PIO", "PADDT");
        }

        bel.add_wire_rc_io("IOLDO_PIO", format!("IOLDO{abcd}_PIO"))
            .add_wire_rc_io("IOLDO_IOLOGIC", format!("IOLDO{abcd}_{iol}"))
            .claim_pip("IOLDO_PIO", "IOLDO_IOLOGIC")
            .add_wire_rc_io("IOLTO_PIO", format!("IOLTO{abcd}_PIO"))
            .add_wire_rc_io("IOLTO_IOLOGIC", format!("IOLTO{abcd}_{iol}"))
            .claim_pip("IOLTO_PIO", "IOLTO_IOLOGIC");

        for (pin, bslot_rc, pin_src) in [
            ("PG", bslots::BCPG, "PGENO_OUT"),
            ("INRD", bslots::BCINRD, "INRDENO_OUT"),
            ("LVDS", bslots::BCLVDSO, "LVDSENO_OUT"),
            ("SLEWRATE", bslots::BCSLEWRATE, "SLEWRATEENO_OUT"),
        ] {
            let cell_rc = bel.ctx.chip.special_loc[&SpecialLocKey::Bc(bank)];
            let bcrd_rc = cell_rc.bel(bslot_rc);
            if !bel.ctx.edev.has_bel(bcrd_rc) {
                continue;
            }
            let wire_src = bel.ctx.naming.bel_wire(bcrd_rc, pin_src);
            let wire = bel.add_wire_rc_io_get(
                pin,
                &if pin == "SLEWRATE" {
                    format!("J{pin}{abcd}_PIO")
                } else {
                    format!("{pin}{abcd}_PIO")
                },
            );
            bel.ctx.claim_pip(wire, wire_src);
        }

        if !is_smol && matches!(bank, 0 | 2) && matches!(idx, 0 | 2) {
            bel.input_xlat_rc_io(bcls::IO::ECLK, format!("ECLK{abcd}_{iol}"));
        }

        if !is_smol && bank != 0 {
            // dummy
            bel.add_wire_rc_io("LVDS", format!("LVDS{abcd}_PIO"));
        }

        if iol == "RIOLOGIC" {
            for pin in ["DQSW90", "DQSR90", "DDRCLKPOL"] {
                let wire = bel.add_wire_rc_io_get(pin, format!("{pin}{abcd}_{iol}"));
                let bel_dqs = bel.ctx.edev.bel_io_dqs(bcrd).unwrap();
                let wire_dqs = bel.ctx.naming.bel_wire(bel_dqs, &format!("{pin}_OUT"));
                bel.ctx.claim_pip(wire, wire_dqs);
            }
        }

        if matches!(
            bel.ctx.chip.kind,
            ChipKind::MachXo2(
                MachXo2Kind::MachXo3Lfp | MachXo2Kind::MachXo3D | MachXo2Kind::MachNx
            )
        ) {
            let is_i3c = edge == Dir::W
                && bel.ctx.chip.rows[cell.row].io[DirH::W] == IoGroupKind::QuadI3c
                && idx < 2;

            if is_i3c {
                bel.input_xlat_rc_io(bcls::IO::RESEN, format!("JRESEN{abcd}_PIO"))
                    .input_xlat_rc_io(bcls::IO::PULLUPEN, format!("JPULLUPEN{abcd}_PIO"));
            } else {
                bel.add_wire_rc_io("RESEN", format!("JRESEN{abcd}_PIO"))
                    .add_wire_rc_io("PULLUPEN", format!("JPULLUPEN{abcd}_PIO"));
            }
        }
    }

    pub(super) fn process_io_machxo2(&mut self) {
        let is_smol = self.chip.rows[self.chip.row_clk].kind != RowKind::Ebr;
        for (row, rd) in &self.chip.rows {
            let num_io = match rd.io[DirH::W] {
                IoGroupKind::Quad | IoGroupKind::QuadI3c => 4,
                IoGroupKind::Double => 2,
                _ => 0,
            };
            for iob in 0..num_io {
                let bcrd = Chip::DIE.cell(self.chip.col_w(), row).bel(bslots::IO[iob]);
                self.process_single_io_machxo2(bcrd);
            }
            let num_io = match rd.io[DirH::E] {
                IoGroupKind::Quad => 4,
                IoGroupKind::Double => 2,
                _ => 0,
            };
            for iob in 0..num_io {
                let bcrd = Chip::DIE.cell(self.chip.col_e(), row).bel(bslots::IO[iob]);
                self.process_single_io_machxo2(bcrd);
            }
        }
        for (col, cd) in &self.chip.columns {
            {
                let cell = Chip::DIE.cell(col, self.chip.row_s());
                let num_io = match cd.io[DirV::S] {
                    IoGroupKind::Quad | IoGroupKind::QuadReverse => 4,
                    IoGroupKind::Double => 2,
                    _ => 0,
                };
                if !is_smol && num_io != 0 {
                    let bcrd_eclksync = self.chip.bel_eclksync(Dir::S, 0);
                    let eclk0 = self.naming.bel_wire(bcrd_eclksync, "ECLKO_OUT");
                    let bcrd_eclksync = self.chip.bel_eclksync(Dir::S, 1);
                    let eclk1 = self.naming.bel_wire(bcrd_eclksync, "ECLKO_OUT");
                    self.extract_switchbox(cell.bel(bslots::IO_INT))
                        .output_bind_rc_io("ECLKA", wires::IMUX_IO_ECLK[0].cell(0))
                        .output_bind_rc_io("ECLKC", wires::IMUX_IO_ECLK[1].cell(0))
                        .input_xlat(eclk0)
                        .input_xlat(eclk1);
                }
                for iob in 0..num_io {
                    let bcrd = cell.bel(bslots::IO[iob]);
                    self.process_single_io_machxo2(bcrd);
                }
            }
            {
                let cell = Chip::DIE.cell(col, self.chip.row_n());
                let num_io = match cd.io[DirV::N] {
                    IoGroupKind::Quad | IoGroupKind::QuadReverse => 4,
                    IoGroupKind::Double => 2,
                    _ => 0,
                };
                if !is_smol && num_io != 0 {
                    let bcrd_eclksync = self.chip.bel_eclksync(Dir::N, 0);
                    let eclk0 = self.naming.bel_wire(bcrd_eclksync, "ECLKO_OUT");
                    let bcrd_eclksync = self.chip.bel_eclksync(Dir::N, 1);
                    let eclk1 = self.naming.bel_wire(bcrd_eclksync, "ECLKO_OUT");
                    self.extract_switchbox(cell.bel(bslots::IO_INT))
                        .output_bind_rc_io("ECLKA", wires::IMUX_IO_ECLK[0].cell(0))
                        .output_bind_rc_io("ECLKC", wires::IMUX_IO_ECLK[1].cell(0))
                        .input_xlat(eclk0)
                        .input_xlat(eclk1);
                }
                for iob in 0..num_io {
                    let bcrd = cell.bel(bslots::IO[iob]);
                    self.process_single_io_machxo2(bcrd);
                }
            }
        }
    }

    pub(super) fn process_clkdiv_machxo2(&mut self) {
        let is_smol = self.chip.rows[self.chip.row_clk].kind != RowKind::Ebr;
        if is_smol {
            return;
        }
        for edge in [DirV::S, DirV::N] {
            for idx in 0..2 {
                let bcrd = self
                    .chip
                    .bel_eclksync(Dir::V(edge), 0)
                    .bel(bslots::CLKDIV[idx]);

                self.hybrid_out_rc(
                    bcrd,
                    format!("JCDIV1{idx}_CLKDIV"),
                    wires::OUT_CLKDIV_CDIV1[idx].cell(0),
                );
                self.hybrid_out_rc(
                    bcrd,
                    format!("JCDIVX{idx}_CLKDIV"),
                    wires::OUT_CLKDIV_CDIVX[idx].cell(0),
                );

                self.extract_bel(bcrd)
                    .name(format!(
                        "{bt}CLKDIV{idx}",
                        bt = match edge {
                            DirV::S => 'B',
                            DirV::N => 'T',
                        }
                    ))
                    .input_xlat_rc(bcls::CLKDIV::CLKI, format!("CLKI{idx}_CLKDIV"))
                    .input_xlat_rc(bcls::CLKDIV::RST, format!("JRST{idx}_CLKDIV"))
                    .input_xlat_rc(bcls::CLKDIV::ALIGNWD, format!("JALIGNWD{idx}_CLKDIV"))
                    .output_xlat_rc(bcls::CLKDIV::CDIV1, format!("JCDIV1{idx}_CLKDIV"))
                    .output_xlat_rc(bcls::CLKDIV::CDIVX, format!("JCDIVX{idx}_CLKDIV"));
            }
        }
    }

    pub(super) fn process_icc_machxo2(&mut self) {
        if self.chip.kind != ChipKind::MachXo2(MachXo2Kind::MachNx) {
            return;
        }
        // fake loc, fake bel.
        let bcrd = self.chip.special_loc[&SpecialLocKey::Config].bel(bslots::IO[0]);
        self.name_bel(bcrd, ["ICC_R20", "ICCREG_R21", "ICC1_R14"]);
        let cell_icc1 = Chip::DIE.cell(ColId::from_idx(0), RowId::from_idx(16));
        for (pin, row, iob) in [
            ("PFRMCLK", 14, 0),
            ("PFRMISO", 13, 1),
            ("PFRMOSI", 13, 0),
            ("PFRCSN", 12, 1),
        ] {
            let row = RowId::from_idx(row);
            let cell_io = cell_icc1.with_row(row);

            let wire_i = self.rc_io_wire(cell_icc1, &format!("J{pin}I_ICC1"));
            self.add_bel_wire(bcrd, format!("{pin}I"), wire_i);
            let wire_int = cell_io.wire(wires::OUT_Q[iob]);
            self.claim_pip_int_out(wire_int, wire_i);

            let wire_o = self.rc_io_wire(cell_icc1, &format!("J{pin}O_ICC1"));
            self.add_bel_wire(bcrd, format!("{pin}O"), wire_o);
            let wire_int = cell_io.wire(wires::IMUX_A[iob]);
            self.claim_pip_int_in(wire_o, wire_int);

            let wire_oe = self.rc_io_wire(cell_icc1, &format!("J{pin}OE_ICC1"));
            self.add_bel_wire(bcrd, format!("{pin}OE"), wire_oe);
            let wire_int = cell_io.wire(wires::IMUX_C[iob]);
            self.claim_pip_int_in(wire_oe, wire_int);
        }
        {
            let (pin, row, iob) = ("PFRMCSN", 16, 1);
            let row = RowId::from_idx(row);
            let cell_io = cell_icc1.with_row(row);
            let wire = self.rc_io_wire(cell_icc1, &format!("J{pin}_ICC1"));
            self.add_bel_wire(bcrd, pin, wire);
            let wire_int = cell_io.wire(wires::OUT_Q[iob]);
            self.claim_pip_int_out(wire_int, wire);
        }
        for (pin, row, iob) in [("PFRPRMN", 11, 1), ("PFRJTAGEN", 11, 2)] {
            let row = RowId::from_idx(row);
            let cell_io = cell_icc1.with_row(row);
            let wire = self.rc_io_wire(cell_icc1, &format!("J{pin}_ICC1"));
            self.add_bel_wire(bcrd, pin, wire);
            let wire_int = cell_io.wire(wires::IMUX_A[iob]);
            self.claim_pip_int_in(wire, wire_int);
        }
        let cell_icc = Chip::DIE.cell(ColId::from_idx(0), RowId::from_idx(10));
        let cell_iccreg = Chip::DIE.cell(ColId::from_idx(0), RowId::from_idx(9));
        for (pin_icc, pin_iccreg, row, iob) in [
            ("TXVIO0", "TXV0", 3, 2),
            ("TXVIO1", "TXV1", 3, 3),
            ("AUX", "TXAUX", 2, 3),
            ("INT", "TXINT", 2, 2),
            ("TXDATA0", "TXD0", 5, 2),
            ("TXDATA1", "TXD1", 5, 3),
            ("TXDATA2", "TXD2", 4, 0),
            ("TXDATA3", "TXD3", 4, 1),
            ("TXDATA4", "TXD4", 4, 2),
            ("TXDATA5", "TXD5", 4, 3),
            ("TXDATA6", "TXD6", 3, 1),
            ("TXDATA7", "TXD7", 10, 0),
            ("TXCLK", "TXCLK", 10, 2),
            ("TXVALID", "TXVLD", 5, 1),
            ("TXREADY", "TXRDY", 5, 0),
        ] {
            let wire_icc = self.rc_io_wire(cell_icc, &format!("J{pin_icc}_ICC"));
            self.add_bel_wire(bcrd, pin_icc, wire_icc);
            let wire_q = self.rc_io_wire(cell_iccreg, &format!("JQ{pin_iccreg}_ICCREG"));
            self.add_bel_wire(bcrd, format!("Q_{pin_iccreg}"), wire_q);
            let wire_d = self.rc_io_wire(cell_iccreg, &format!("JD{pin_iccreg}_ICCREG"));
            self.add_bel_wire(bcrd, format!("D_{pin_iccreg}"), wire_d);
            let row = RowId::from_idx(row);
            let cell_io = cell_icc1.with_row(row);
            let wire_int = cell_io.wire(wires::IMUX_A[iob]);
            self.claim_pip_int_in(wire_icc, wire_int);
            self.claim_pip_int_in(wire_d, wire_int);
            self.claim_pip(wire_icc, wire_q);

            let wire_clk = self.rc_io_wire(cell_iccreg, &format!("JCK{pin_iccreg}_ICCREG"));
            self.add_bel_wire(bcrd, format!("CLK_{pin_iccreg}"), wire_clk);
            let wire_int = cell_io.wire(wires::IMUX_CLK[iob]);
            self.claim_pip_int_in(wire_clk, wire_int);

            let wire_rst = self.rc_io_wire(cell_iccreg, &format!("JRS{pin_iccreg}_ICCREG"));
            self.add_bel_wire(bcrd, format!("RST_{pin_iccreg}"), wire_rst);
            let wire_int = cell_io.wire(wires::IMUX_LSR[iob]);
            self.claim_pip_int_in(wire_rst, wire_int);

            let wire_ce = self.rc_io_wire(cell_iccreg, &format!("JCE{pin_iccreg}_ICCREG"));
            self.add_bel_wire(bcrd, format!("CE_{pin_iccreg}"), wire_ce);
            let wire_int = cell_io.wire(wires::IMUX_CE[iob]);
            self.claim_pip_int_in(wire_ce, wire_int);
            if pin_icc == "TXCLK" {
                // ???
                let wire_int = cell_io.wire(wires::IMUX_B[iob]);
                self.claim_pip_int_in(wire_ce, wire_int);
            }
        }
        for (pin_icc, pin_iccreg, row, iob) in [
            ("RXVIO0", "RXV0", 2, 0),
            ("RXVIO1", "RXV1", 2, 1),
            ("RXDATA0", "RXD0", 10, 3),
            ("RXDATA1", "RXD1", 9, 0),
            ("RXDATA2", "RXD2", 9, 1),
            ("RXDATA3", "RXD3", 9, 2),
            ("RXDATA4", "RXD4", 9, 3),
            ("RXDATA5", "RXD5", 6, 0),
            ("RXDATA6", "RXD6", 6, 1),
            ("RXDATA7", "RXD7", 6, 2),
            ("RXVALID", "RXVLD", 6, 3),
            ("RXREADY", "RXRDY", 10, 1),
        ] {
            let wire_icc = self.rc_io_wire(cell_icc, &format!("J{pin_icc}_ICC"));
            self.add_bel_wire(bcrd, pin_icc, wire_icc);
            let wire_q = self.rc_io_wire(cell_iccreg, &format!("JQ{pin_iccreg}_ICCREG"));
            self.add_bel_wire(bcrd, format!("Q_{pin_iccreg}"), wire_q);
            let wire_d = self.rc_io_wire(cell_iccreg, &format!("JD{pin_iccreg}_ICCREG"));
            self.add_bel_wire(bcrd, format!("D_{pin_iccreg}"), wire_d);
            let row = RowId::from_idx(row);
            let cell_io = cell_icc1.with_row(row);
            self.claim_pip(wire_d, wire_icc);
            let wire_int = cell_io.wire(wires::OUT_F[iob]);
            self.claim_pip_int_out(wire_int, wire_q);
            let wire_int = cell_io.wire(wires::OUT_Q[iob]);
            self.claim_pip_int_out(wire_int, wire_icc);

            let wire_clk = self.rc_io_wire(cell_iccreg, &format!("JCK{pin_iccreg}_ICCREG"));
            self.add_bel_wire(bcrd, format!("CLK_{pin_iccreg}"), wire_clk);
            let wire_int = cell_io.wire(wires::IMUX_CLK[iob]);
            self.claim_pip_int_in(wire_clk, wire_int);

            let wire_rst = self.rc_io_wire(cell_iccreg, &format!("JRS{pin_iccreg}_ICCREG"));
            self.add_bel_wire(bcrd, format!("RST_{pin_iccreg}"), wire_rst);
            let wire_int = cell_io.wire(wires::IMUX_LSR[iob]);
            self.claim_pip_int_in(wire_rst, wire_int);

            let wire_ce = self.rc_io_wire(cell_iccreg, &format!("JCE{pin_iccreg}_ICCREG"));
            self.add_bel_wire(bcrd, format!("CE_{pin_iccreg}"), wire_ce);
            let wire_int = cell_io.wire(wires::IMUX_CE[iob]);
            self.claim_pip_int_in(wire_ce, wire_int);
        }

        let wire_icc = self.rc_io_wire(cell_icc, "JRXCLK_ICC");
        self.add_bel_wire(bcrd, "RXCLK", wire_icc);
        let row = RowId::from_idx(3);
        let cell_io = cell_icc1.with_row(row);
        let wire_int = cell_io.wire(wires::OUT_Q[0]);
        self.claim_pip_int_out(wire_int, wire_icc);
        let cell_dlldel = cell_icc1.with_row(self.chip.row_clk);
        let wire_clki = self.rc_io_wire(cell_dlldel, "JCLKI0_DLLDEL");
        let wire_paddi = self.rc_io_wire(cell_dlldel, "JPADDI0");
        self.claim_pip(wire_clki, wire_icc);
        self.claim_pip(wire_paddi, wire_icc);
    }
}
