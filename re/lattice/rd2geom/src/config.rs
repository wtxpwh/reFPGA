use prjcombine_ecp::{
    chip::{Chip, ChipKind, MachXo2Kind, SpecialIoKey, SpecialLocKey},
    defs::{bcls, bslots, tcls, wires},
};
use prjcombine_interconnect::{
    db::WireSlotIdExt,
    dir::{DirH, DirHV, DirV},
    grid::DieIdExt,
};
use prjcombine_re_lattice_naming::WireName;

use crate::ChipContext;

impl ChipContext<'_> {
    fn process_config_scm(&mut self) {
        let cell = self.chip.special_loc[&SpecialLocKey::Config];

        let cell_nw = cell.with_cr(self.chip.col_w(), self.chip.row_n());
        let cell_ne = cell.with_cr(self.chip.col_e(), self.chip.row_n());

        self.extract_bel(cell.bel(bslots::START))
            .name("START")
            .suffix("START");

        self.extract_bel(cell.bel(bslots::RDBK))
            .name("RDBK")
            .suffix("RDBK");

        self.extract_bel(cell.bel(bslots::OSC))
            .name("OSC")
            .suffix("OSC");

        {
            let rstn = self.rc_wire(cell, "JRSTN_GSR");
            let rstn_io = self.rc_corner_wire(cell_nw, "JRSTN_RSTN");
            self.claim_pip(rstn, rstn_io);

            let usr = self.rc_wire(cell, "JUSR_GSR");
            let gsr = self.rc_wire(cell, "GSR_GSR");
            self.claim_pip(gsr, rstn);
            self.claim_pip(gsr, usr);

            self.extract_bel(cell.bel(bslots::GSR))
                .name("GSR")
                .suffix("GSR")
                .add_wire("RSTN", rstn)
                .add_wire("GSR", gsr);
        }

        {
            let bcrd = cell.bel(bslots::JTAG);
            let cell_jtag = cell.delta(1, 0);
            let name = if self.chip.rows.len() == 58 {
                // what the fuck.
                "JTAG_R13C26"
            } else {
                "JTAG"
            };

            let tdo = self.rc_wire(cell_jtag, "TDO_JTAG");
            let tdo_mux = self.rc_wire(cell_jtag, "JTDO");
            let tdo_int = self.rc_wire(cell_jtag, "JTDOCIB");
            let tdo_io = self.rc_corner_wire(cell_ne, "JTDO_TDO");
            self.claim_pip(tdo_mux, tdo);
            self.claim_pip(tdo_mux, tdo_int);
            self.claim_pip(tdo_io, tdo_mux);

            let mut bel = self.extract_bel_cell(bcrd, cell_jtag);
            bel.names([name, "TDO"])
                .suffix("JTAG")
                .add_wire("TDO", tdo)
                .add_wire("TDO_MUX", tdo_mux)
                .add_wire("TDO_INT", tdo_int)
                .add_wire("TDO_IO", tdo_io)
                .input_xlat_no_name(bcls::JTAG::TDO, tdo_int);

            for pin in [bcls::JTAG::TCK, bcls::JTAG::TMS, bcls::JTAG::TDI] {
                let pname = bel.intdb[bcls::JTAG].outputs.key(pin).0;
                let wire = bel.ctx.rc_wire(cell_jtag, &format!("J{pname}_JTAG"));
                bel.add_wire(pname, wire);
                let wire_io = bel
                    .ctx
                    .rc_corner_wire(cell_ne, &format!("J{pname}_{pname}"));
                bel.ctx.claim_pip(wire, wire_io);
                bel.output_xlat_quiet_no_name(pin, wire_io);
            }
        }

        {
            let bcrd = cell.bel(bslots::SYSBUS);
            let cell_sysbus = cell.delta(2, 0);
            let mut bel = self.extract_bel_cell(bcrd, cell_sysbus);
            bel.names(["SYSBUS", "MPIIRQN"]).suffix("SYSBUS");
            for pin in [
                "LPCSQ0",
                "LPCSQ1",
                "LPCSQ2",
                "LPCSQ3",
                "RPCSQ0",
                "RPCSQ1",
                "RPCSQ2",
                "RPCSQ3",
                "PD20TS",
                "PD73TS",
                "PD158TS",
                "PD3116TS",
                "MPIDPTS0",
                "MPIDPTS1",
                "MPIDPTS2",
                "MPICNTLTS",
            ] {
                bel.add_wire_rc(pin, format!("J{pin}_SYSBUS"));
            }
            for i in 0..32 {
                let io = bel.ctx.chip.special_io[&SpecialIoKey::D(i)];
                let (cell_io, abcd) = bel.ctx.xlat_io_loc_scm(io);

                let rddata =
                    bel.add_wire_rc_get(format!("MPIRDDATA{i}"), format!("JMPIRDDATA{i}_SYSBUS"));
                let paddo = bel.ctx.rc_io_wire(cell_io, &format!("JPADDO{abcd}_PIO"));
                bel.ctx.claim_pip(paddo, rddata);

                let pdts = bel.ctx.rc_wire(
                    cell_sysbus,
                    match i {
                        0..3 => "JPD20TS_SYSBUS",
                        3..8 => "JPD73TS_SYSBUS",
                        8..16 => "JPD158TS_SYSBUS",
                        16.. => "JPD3116TS_SYSBUS",
                    },
                );
                let paddt = bel.ctx.rc_io_wire(cell_io, &format!("JPADDT{abcd}_PIO"));
                bel.ctx.claim_pip(paddt, pdts);

                let wrdata =
                    bel.add_wire_rc_get(format!("MPIWRDATA{i}"), format!("JMPIWRDATA{i}_SYSBUS"));
                let inddck = bel.ctx.rc_io_wire(cell_io, &format!("JINDDCK{abcd}"));
                bel.ctx.claim_pip(wrdata, inddck);
            }
            for i in 0..4 {
                let io = bel.ctx.chip.special_io[&SpecialIoKey::DP(i)];
                let (cell_io, abcd) = bel.ctx.xlat_io_loc_scm(io);

                let rddata = bel
                    .add_wire_rc_get(format!("MPIRDPARITY{i}"), format!("JMPIRDPARITY{i}_SYSBUS"));
                let paddo = bel.ctx.rc_io_wire(cell_io, &format!("JPADDO{abcd}_PIO"));
                bel.ctx.claim_pip(paddo, rddata);

                let pdts = bel.ctx.rc_wire(
                    cell_sysbus,
                    match i {
                        0 => "JMPIDPTS0_SYSBUS",
                        1 => "JMPIDPTS1_SYSBUS",
                        _ => "JMPIDPTS2_SYSBUS",
                    },
                );
                let paddt = bel.ctx.rc_io_wire(cell_io, &format!("JPADDT{abcd}_PIO"));
                bel.ctx.claim_pip(paddt, pdts);

                let wrdata = bel
                    .add_wire_rc_get(format!("MPIWRPARITY{i}"), format!("JMPIWRPARITY{i}_SYSBUS"));
                let inddck = bel.ctx.rc_io_wire(cell_io, &format!("JINDDCK{abcd}"));
                bel.ctx.claim_pip(wrdata, inddck);
            }

            let mpicntlts = bel.ctx.rc_wire(cell_sysbus, "JMPICNTLTS_SYSBUS");
            for (pin, key) in [
                ("MPITEA", SpecialIoKey::MpiTeaN),
                ("MPITA", SpecialIoKey::MpiAckN),
                ("MPIRETRY", SpecialIoKey::MpiRetryN),
            ] {
                let wire = bel.add_wire_rc_get(pin, format!("J{pin}_SYSBUS"));

                let io = bel.ctx.chip.special_io[&key];
                let (cell_io, abcd) = bel.ctx.xlat_io_loc_scm(io);

                let paddo = bel.ctx.rc_io_wire(cell_io, &format!("JPADDO{abcd}_PIO"));
                bel.ctx.claim_pip(paddo, wire);
                let paddt = bel.ctx.rc_io_wire(cell_io, &format!("JPADDT{abcd}_PIO"));
                bel.ctx.claim_pip(paddt, mpicntlts);
            }
            for (pin, key) in [
                ("EXTDONEO", SpecialIoKey::ExtDoneO),
                ("EXTCLKP1O", SpecialIoKey::ExtClkO(1)),
                ("EXTCLKP2O", SpecialIoKey::ExtClkO(2)),
            ] {
                let wire = bel.add_wire_rc_get(pin, format!("J{pin}_SYSBUS"));

                let io = bel.ctx.chip.special_io[&key];
                let (cell_io, abcd) = bel.ctx.xlat_io_loc_scm(io);

                let paddo = bel.ctx.rc_io_wire(cell_io, &format!("JPADDO{abcd}_PIO"));
                bel.ctx.claim_pip(paddo, wire);
            }
            for (pin, key) in [
                ("CS0N", SpecialIoKey::CsN),
                ("CS1", SpecialIoKey::Cs1),
                ("MPISTRBN", SpecialIoKey::ReadN),
                ("MPIRWN", SpecialIoKey::WriteN),
                ("MPICLK", SpecialIoKey::MpiClk),
                ("MPIADDR14", SpecialIoKey::A(0)),
                ("MPIADDR15", SpecialIoKey::A(1)),
                ("MPIADDR16", SpecialIoKey::A(2)),
                ("MPIADDR17", SpecialIoKey::A(3)),
                ("MPIADDR18", SpecialIoKey::A(4)),
                ("MPIADDR19", SpecialIoKey::A(5)),
                ("MPIADDR20", SpecialIoKey::A(6)),
                ("MPIADDR21", SpecialIoKey::A(7)),
                ("MPIADDR22", SpecialIoKey::A(8)),
                ("MPIADDR23", SpecialIoKey::A(9)),
                ("MPIADDR24", SpecialIoKey::A(10)),
                ("MPIADDR25", SpecialIoKey::A(11)),
                ("MPIADDR26", SpecialIoKey::A(12)),
                ("MPIADDR27", SpecialIoKey::A(13)),
                ("MPIADDR28", SpecialIoKey::A(14)),
                ("MPIADDR29", SpecialIoKey::A(15)),
                ("MPIADDR30", SpecialIoKey::A(16)),
                ("MPIADDR31", SpecialIoKey::A(17)),
                ("MPITSIZ0", SpecialIoKey::A(18)),
                ("MPITSIZ1", SpecialIoKey::A(19)),
                ("MPIBDIP", SpecialIoKey::A(20)),
                ("MPIBURST", SpecialIoKey::A(21)),
                ("EXTDONEI", SpecialIoKey::ExtDoneI),
                ("EXTCLKP1I", SpecialIoKey::ExtClkI(1)),
                ("EXTCLKP2I", SpecialIoKey::ExtClkI(2)),
            ] {
                let wire = bel.add_wire_rc_get(pin, format!("J{pin}_SYSBUS"));

                let io = bel.ctx.chip.special_io[&key];
                let (cell_io, abcd) = bel.ctx.xlat_io_loc_scm(io);

                let inddck = bel.ctx.rc_io_wire(cell_io, &format!("JINDDCK{abcd}"));
                bel.ctx.claim_pip(wire, inddck);
            }

            for (pin, pname) in [
                (bcls::SYSBUS::MPIRSTN, "MPIRSTN"),
                (bcls::SYSBUS::SYSRSTN, "SYSRSTN"),
            ] {
                let wire_rstn =
                    bel.add_wire_rc_get(format!("{pname}_RSTN"), format!("JRSTN_{pname}"));
                bel.add_wire_rc(pname, format!("{pname}_SYSBUS"))
                    .input_xlat_rename_rc(pin, format!("{pname}_INT"), format!("JUSR_{pname}"))
                    .claim_pip(pname, format!("{pname}_INT"))
                    .claim_pip(pname, format!("{pname}_RSTN"));

                let wire_io = bel.ctx.rc_corner_wire(cell_nw, "JRSTN_RSTN");
                bel.ctx.claim_pip(wire_rstn, wire_io);
            }

            let mpiirq_out = bel.ctx.rc_corner_wire(cell_ne, "JMPIIRQN_MPIIRQN");
            bel.add_wire_rc("MPIIRQN", "JMPIIRQN_SYSBUS")
                .add_wire("MPIIRQN_OUT", mpiirq_out)
                .claim_pip("MPIIRQN_OUT", "MPIIRQN");
        }
    }

    fn process_config_ecp(&mut self) {
        let chip = self.chip;
        let cell = chip.special_loc[&SpecialLocKey::Config];

        let mut bel = self.extract_bel(cell.bel(bslots::JTAG));
        bel.names(["JTAG", "TCK", "TMS", "TDI", "TDO"])
            .suffix("JTAG");
        for pin in ["TCK", "TMS", "TDI", "TDO"] {
            let wire = bel.ctx.rc_wire(cell, &format!("J{pin}_JTAG"));
            let wire_pin = WireName {
                r: 0,
                c: chip.columns.len() as u8 + 1,
                suffix: bel
                    .ctx
                    .naming
                    .strings
                    .get(&format!("J{pin}_{pin}"))
                    .unwrap(),
            };
            bel.add_wire(pin, wire);
            bel.add_wire(format!("{pin}_{pin}"), wire_pin);
            if pin == "TDO" {
                bel.ctx.claim_pip(wire_pin, wire);
            } else {
                bel.ctx.claim_pip(wire, wire_pin);
            }
        }
        core::mem::drop(bel);

        {
            let mut bel = self.extract_bel(cell.bel(bslots::START));

            if chip.kind == ChipKind::Ecp {
                bel.name("START");
            } else {
                bel.names(["START", "OSC"]);
                let wire = bel.ctx.rc_wire(cell, "JCFGCLK_OSC");
                bel.add_wire("CFGCLK_OSC", wire);
            }

            bel.suffix("START");
        }

        let bcrd = cell.bel(bslots::RDBK);
        self.name_bel(bcrd, ["RDBK"]);
        for (pin, wire) in [
            ("CAPTINPUT", "JCAPTINPUT_RDBK"),
            ("CAPTCLK", "JCAPTCLK_RDBK"),
        ] {
            let wire = self.rc_wire(cell, wire);
            self.add_bel_wire(bcrd, pin, wire);
        }

        if chip.kind == ChipKind::Ecp {
            self.extract_bel(cell.bel(bslots::OSC))
                .name("OSC")
                .suffix("OSC");
        }

        self.extract_bel(cell.bel(bslots::GSR))
            .name("GSR")
            .suffix("GSR");
    }

    fn process_config_machxo(&mut self) {
        let chip = self.chip;
        let cell = chip.special_loc[&SpecialLocKey::Config];

        let mut bel = self.extract_bel(cell.bel(bslots::JTAG));
        bel.names(["JTAG", "TCK", "TMS", "TDI", "TDO"])
            .suffix("JTAG");
        for pin in ["TCK", "TMS", "TDI", "TDO"] {
            let wire = bel.ctx.rc_wire(cell, &format!("J{pin}_JTAG"));
            let wire_pin = WireName {
                r: chip.rows.len() as u8 + 1,
                c: 0,
                suffix: bel
                    .ctx
                    .naming
                    .strings
                    .get(&format!("J{pin}_{pin}"))
                    .unwrap(),
            };
            bel.add_wire(pin, wire);
            bel.add_wire(format!("{pin}_{pin}"), wire_pin);
            if pin == "TDO" {
                bel.ctx.claim_pip(wire_pin, wire);
            } else {
                bel.ctx.claim_pip(wire, wire_pin);
            }
        }
        core::mem::drop(bel);

        let tsall_io = chip.special_io[&SpecialIoKey::TsAll];
        let tsalli = self.rc_wire(tsall_io.cell, "JTSALLI_TSALL");
        let wire_io = self.get_special_io_wire_in(SpecialIoKey::TsAll);
        self.claim_pip(tsalli, wire_io);
        self.extract_bel(cell.bel(bslots::TSALL))
            .name("TSALL")
            .add_wire("TSALLI", tsalli);

        let gsrpadn = self.rc_wire(cell, "JGSRPADN_GSR");
        let wire_io = self.get_special_io_wire_in(SpecialIoKey::Gsr);
        self.claim_pip(gsrpadn, wire_io);
        self.extract_bel(cell.bel(bslots::GSR))
            .name("GSR")
            .suffix("GSR")
            .add_wire("GSRPADN", gsrpadn);
    }

    fn process_osc_machxo(&mut self) {
        let cell = self.chip.special_loc[&SpecialLocKey::Osc];
        self.extract_bel(cell.bel(bslots::OSC))
            .name("OSC")
            .suffix("OSC");
    }

    fn process_config_ecp2(&mut self) {
        let chip = self.chip;
        let cell = chip.special_loc[&SpecialLocKey::Config];

        let mut bel = self.extract_bel(cell.bel(bslots::JTAG));
        bel.names(["JTAG", "TCK", "TMS", "TDI", "TDO"])
            .suffix("JTAG");
        for pin in ["TCK", "TMS", "TDI", "TDO"] {
            let wire = bel.ctx.rc_wire(cell, &format!("J{pin}_JTAG"));
            let wire_pin = WireName {
                r: chip.rows.len() as u8 + 1,
                c: 0,
                suffix: bel
                    .ctx
                    .naming
                    .strings
                    .get(&format!("J{pin}_{pin}"))
                    .unwrap(),
            };
            bel.add_wire(pin, wire);
            bel.add_wire(format!("{pin}_{pin}"), wire_pin);
            if pin == "TDO" {
                bel.ctx.claim_pip(wire_pin, wire);
            } else {
                bel.ctx.claim_pip(wire, wire_pin);
            }
        }
        core::mem::drop(bel);

        self.extract_bel(cell.bel(bslots::START))
            .name("START")
            .suffix("START");

        self.extract_bel(cell.bel(bslots::OSC))
            .name("OSC")
            .suffix("OSC");

        self.extract_bel(cell.bel(bslots::GSR))
            .name("GSR")
            .suffix("GSR");

        self.extract_bel(cell.bel(bslots::SED))
            .name("SED")
            .suffix("SED");

        self.extract_bel(cell.bel(bslots::SPIM))
            .name("SPIM")
            .suffix("SPIM");
    }

    fn process_config_xp2(&mut self) {
        let cell = self.chip.special_loc[&SpecialLocKey::Osc];
        self.extract_bel_cell(cell.bel(bslots::OSC), cell.delta(0, 1))
            .name("OSC")
            .suffix("OSC");

        let cell = self.chip.clk_emid(DirH::E);

        self.extract_bel(cell.bel(bslots::WAKEUP))
            .name("WAKEUP")
            .suffix("WAKEUP");

        self.extract_bel(cell.bel(bslots::GSR))
            .name("GSR")
            .suffix("GSR");

        self.extract_bel(cell.bel(bslots::STF))
            .name("STF")
            .suffix("STF");

        self.extract_bel(cell.bel(bslots::START))
            .name("START")
            .suffix("START");

        {
            let mut bel = self.extract_bel(cell.bel(bslots::SSPI));
            bel.names(["SSPICIB", "SSPIPIN"]).suffix("SSPICIB");
            for (pin, key) in [
                ("CLK", SpecialIoKey::Cclk),
                ("CS", SpecialIoKey::SpiPCsB),
                ("SI", SpecialIoKey::D(0)),
                ("SO", SpecialIoKey::D(1)),
            ] {
                let wire = bel.ctx.rc_wire(cell, &format!("J{pin}_SSPIPIN"));
                bel.add_wire(format!("{pin}_IO"), wire);
                if pin == "SO" {
                    let wire_io = bel.ctx.get_special_io_wire_out(key);
                    bel.ctx.claim_pip(wire_io, wire);
                } else {
                    let wire_io = bel.ctx.get_special_io_wire_in(key);
                    bel.ctx.claim_pip(wire, wire_io);
                }
            }
        }

        let cell_cfg = self.chip.special_loc[&SpecialLocKey::Config];

        self.extract_bel_cell(cell_cfg.bel(bslots::SED), cell)
            .name("SED")
            .suffix("SED");

        let mut bel = self.extract_bel_cell(cell_cfg.bel(bslots::JTAG), cell);
        bel.names(["JTAG", "TCK", "TMS", "TDI", "TDO"])
            .suffix("JTAG");
        for pin in ["TCK", "TMS", "TDI", "TDO"] {
            let wire = bel.ctx.rc_wire(cell, &format!("J{pin}_JTAG"));
            let wire_pin = bel.ctx.rc_io_wire(cell, &format!("J{pin}_{pin}"));
            bel.add_wire(pin, wire);
            bel.add_wire(format!("{pin}_{pin}"), wire_pin);
            if pin == "TDO" {
                bel.ctx.claim_pip(wire_pin, wire);
            } else {
                bel.ctx.claim_pip(wire, wire_pin);
            }
        }
    }

    fn process_config_ecp3(&mut self) {
        let cell = self.chip.special_loc[&SpecialLocKey::Config];

        let mut bel = self.extract_bel_cell(cell.bel(bslots::JTAG), cell.delta(3, 0));
        bel.names(["JTAG", "TCK", "TMS", "TDI", "TDO"])
            .suffix("JTAG");
        for pin in ["TCK", "TMS", "TDI", "TDO"] {
            let wire = bel.ctx.rc_wire(cell.delta(3, 0), &format!("J{pin}_JTAG"));
            let wire_pin = WireName {
                r: 0,
                c: 0,
                suffix: bel
                    .ctx
                    .naming
                    .strings
                    .get(&format!("J{pin}_{pin}"))
                    .unwrap(),
            };
            bel.add_wire(pin, wire);
            bel.add_wire(format!("{pin}_{pin}"), wire_pin);
            // lmao yes they reversed this
            if pin == "TDO" {
                bel.ctx.claim_pip(wire, wire_pin);
            } else {
                bel.ctx.claim_pip(wire_pin, wire);
            }
        }
        core::mem::drop(bel);

        self.extract_bel_cell(cell.bel(bslots::START), cell.delta(3, 0))
            .name("START")
            .suffix("START");

        self.extract_bel_cell(cell.bel(bslots::OSC), cell.delta(3, 0))
            .name("OSC")
            .suffix("OSC");

        self.extract_bel_cell(cell.bel(bslots::GSR), cell.delta(3, 0))
            .name("GSR")
            .suffix("GSR");

        self.extract_bel_cell(cell.bel(bslots::SED), cell.delta(4, 0))
            .name("SED")
            .suffix("SED");

        self.extract_bel_cell(cell.bel(bslots::AMBOOT), cell.delta(4, 0))
            .name("AMBOOT")
            .suffix("AMBOOT");

        self.extract_bel_cell(cell.bel(bslots::PERREG), cell.delta(5, 0))
            .name("PERREG")
            .suffix("PERREG");

        for h in [DirH::W, DirH::E] {
            let col = self.chip.col_edge(h);
            for v in [DirV::S, DirV::N] {
                let lr = match h {
                    DirH::W => 'L',
                    DirH::E => 'R',
                };
                let lu = match v {
                    DirV::S => 'L',
                    DirV::N => 'U',
                };
                let row = self.chip.row_edge(v);
                let cell = Chip::DIE.cell(col, row);
                for (bslot, name) in [(bslots::TESTIN, "TESTIN"), (bslots::TESTOUT, "TESTOUT")] {
                    self.extract_bel(cell.bel(bslot))
                        .name(format!("{lu}{lr}{name}"))
                        .suffix(name);
                }
                if h == DirH::E && v == DirV::S {
                    self.extract_bel(cell.bel(bslots::DTS))
                        .name("DTS")
                        .suffix("DTS");
                }
            }
        }
    }

    fn process_config_machxo2(&mut self) {
        let cell = self.chip.special_loc[&SpecialLocKey::Config];

        self.extract_switchbox(cell.bel(bslots::BEL_INT))
            .output_bind_rc("PCNTRCLK", wires::IMUX_PCNTR_CLK.cell(0))
            .input_xlat_rc("JOSCCLK")
            .input_xlat_rc("JCIBCLK");

        let mut bel = self.extract_bel(cell.bel(bslots::JTAG));
        bel.name("JTAG").suffix("JTAG");
        for (pin, key) in [
            ("TCK", SpecialIoKey::Tck),
            ("TMS", SpecialIoKey::Tms),
            ("TDI", SpecialIoKey::Tdi),
            ("TDO", SpecialIoKey::Tdo),
        ] {
            let wire = bel.ctx.rc_wire(cell, &format!("J{pin}_JTAG"));
            bel.add_wire(pin, wire);
            if pin == "TDO" {
                let bcrd_io = bel.ctx.chip.special_io[&key];
                let io_idx = bslots::IO.index_of(bcrd_io.slot).unwrap();
                let wire_io = bel.ctx.rc_io_wire(
                    bcrd_io.cell,
                    &format!("JPADDO{abcd}", abcd = ['A', 'B', 'C', 'D'][io_idx],),
                );
                bel.ctx.claim_pip(wire, wire_io);
            } else {
                // yeah they not good at it
                let wire_io = bel.ctx.get_special_io_wire_in(key);
                bel.ctx.claim_pip(wire_io, wire);
            }
        }
        core::mem::drop(bel);

        let bcrd_osc = cell.bel(bslots::OSC);
        self.extract_bel(bcrd_osc).name("OSC").suffix("OSC");

        self.extract_bel(cell.bel(bslots::START))
            .name("START")
            .suffix("START");

        self.extract_bel(cell.bel(bslots::GSR))
            .name("GSR")
            .suffix("GSR");

        let bcrd_efb = cell.bel(bslots::EFB);
        {
            let mut bel = self.extract_bel(bcrd_efb);
            bel.name("EFB").suffix("EFB");
            for (pin, key) in [
                ("UFMSN", SpecialIoKey::SpiPCsB),
                ("SPIMOSII", SpecialIoKey::D(0)),
                ("SPIMISOI", SpecialIoKey::D(1)),
                ("SPISCKI", SpecialIoKey::Cclk),
                ("I2C1SCLI", SpecialIoKey::D(2)),
                ("I2C1SDAI", SpecialIoKey::D(3)),
            ] {
                let wire = bel.add_wire_rc_get(pin, format!("J{pin}_EFB"));
                let wire_io = bel.ctx.get_special_io_wire_in(key);
                bel.ctx.claim_pip(wire, wire_io);
            }
            for (pin_o, pin_oe, key) in [
                ("I2C1SCLO", "I2C1SCLOEN", SpecialIoKey::D(2)),
                ("I2C1SDAO", "I2C1SDAOEN", SpecialIoKey::D(3)),
                ("SPIMCSN0", "SPIMCSN0", SpecialIoKey::SpiCCsB),
                ("SPISCKO", "SPISCKEN", SpecialIoKey::Cclk),
                ("SPIMISOO", "SPIMISOEN", SpecialIoKey::D(1)),
                ("SPIMOSIO", "SPIMOSIEN", SpecialIoKey::D(0)),
            ] {
                let wire_o = bel.add_wire_rc_get(pin_o, format!("J{pin_o}_EFB"));
                let wire_oe = if pin_o != pin_oe {
                    bel.add_wire_rc_get(pin_oe, format!("J{pin_oe}_EFB"))
                } else {
                    wire_o
                };
                let bcrd_io = bel.ctx.chip.special_io[&key];
                let io_idx = bslots::IO.index_of(bcrd_io.slot).unwrap();
                let wire_io_o = bel.ctx.rc_io_wire(
                    bcrd_io.cell,
                    &format!("JPADDO{abcd}", abcd = ['A', 'B', 'C', 'D'][io_idx],),
                );
                bel.ctx.claim_pip(wire_io_o, wire_o);
                let wire_io_oe = bel.ctx.rc_io_wire(
                    bcrd_io.cell,
                    &format!("JPADDT{abcd}", abcd = ['A', 'B', 'C', 'D'][io_idx],),
                );
                bel.ctx.claim_pip(wire_io_oe, wire_oe);
            }
            // bug? idk perhaps.
            bel.add_wire_rc("SPICSNEN", "JSPICSNEN_EFB");

            for pin in [
                "PLLDATO0",
                "PLLDATO1",
                "PLLDATO2",
                "PLLDATO3",
                "PLLDATO4",
                "PLLDATO5",
                "PLLDATO6",
                "PLLDATO7",
                "PLLADRO0",
                "PLLADRO1",
                "PLLADRO2",
                "PLLADRO3",
                "PLLADRO4",
                "PLLWEO",
                "PLLCLKO",
                "PLLRSTO",
                "PLL0STBO",
                "PLL0ACKI",
                "PLL0DATI0",
                "PLL0DATI1",
                "PLL0DATI2",
                "PLL0DATI3",
                "PLL0DATI4",
                "PLL0DATI5",
                "PLL0DATI6",
                "PLL0DATI7",
                "PLL1STBO",
                "PLL1ACKI",
                "PLL1DATI0",
                "PLL1DATI1",
                "PLL1DATI2",
                "PLL1DATI3",
                "PLL1DATI4",
                "PLL1DATI5",
                "PLL1DATI6",
                "PLL1DATI7",
            ] {
                bel.add_wire_rc(pin, format!("J{pin}_EFB"));
            }

            let has_2pll = bel.ctx.chip.try_bel_pll(DirHV::NE, 0).is_some();

            let has_mux = matches!(
                bel.ctx.chip.kind,
                ChipKind::MachXo2(MachXo2Kind::MachXo2 | MachXo2Kind::MachXo3L)
            ) && has_2pll;

            for (pin, pin_pll) in [
                ("ACKI", "ACK"),
                ("DATI0", "DATO0"),
                ("DATI1", "DATO1"),
                ("DATI2", "DATO2"),
                ("DATI3", "DATO3"),
                ("DATI4", "DATO4"),
                ("DATI5", "DATO5"),
                ("DATI6", "DATO6"),
                ("DATI7", "DATO7"),
            ] {
                for (i, hv) in [(0, DirHV::NW), (1, DirHV::NE)] {
                    let Some(bel_pll) = bel.ctx.chip.try_bel_pll(hv, 0) else {
                        continue;
                    };
                    let cell_pll = bel_pll.cell;
                    let wire_pll = bel.ctx.rc_wire(cell_pll, &format!("JPLL{pin_pll}_PLL"));
                    if has_mux {
                        let wire =
                            bel.add_wire_rc_get(format!("PLL{i}{pin}_IN"), format!("JPLL{i}{pin}"));
                        bel.ctx.claim_pip(wire, wire_pll);
                    } else {
                        let wire = bel.ctx.rc_wire(cell, &format!("JPLL{i}{pin}_EFB"));
                        bel.ctx.claim_pip(wire, wire_pll);
                    }
                }
                if has_2pll {
                    for i in 0..2 {
                        let wire = bel.ctx.rc_wire(cell, &format!("JPLL{i}{pin}_EFB"));
                        let wire_mux = bel
                            .add_wire_rc_get(format!("PLL{i}{pin}_MUX"), format!("PLL{i}{pin}MUX"));
                        bel.ctx.claim_pip(wire, wire_mux);
                        if has_mux {
                            for j in 0..2 {
                                let wire_in = bel.ctx.rc_wire(cell, &format!("JPLL{j}{pin}"));
                                bel.ctx.claim_pip(wire_mux, wire_in);
                            }
                        }
                    }
                }
            }

            if has_2pll {
                for i in 0..2 {
                    let wire = bel.ctx.rc_wire(cell, &format!("JPLL{i}STBO_EFB"));
                    let wire_out =
                        bel.add_wire_rc_get(format!("PLL{i}STBO_OUT"), format!("PLL{i}STBO"));
                    bel.ctx.claim_pip(wire_out, wire);
                }
            }
            if has_mux {
                for i in 0..2 {
                    let wire_mux =
                        bel.add_wire_rc_get(format!("PLL{i}STBO_MUX"), format!("JPLL{i}STBOMUX"));
                    for j in 0..2 {
                        let wire_out = bel.ctx.rc_wire(cell, &format!("PLL{j}STBO"));
                        bel.ctx.claim_pip(wire_mux, wire_out);
                    }
                }
            }
        }

        {
            let mut bel = self.extract_bel(cell.bel(bslots::PCNTR));
            bel.name("PCNTR")
                .suffix("PCNTR")
                .input_xlat_rc(bcls::PCNTR::CLK, "CLK_PCNTR");
            for pin in ["CFGWAKE", "CFGSTDBY"] {
                let wire_efb = bel.ctx.rc_wire(cell, &format!("{pin}_EFB"));
                bel.ctx.add_bel_wire(bcrd_efb, pin, wire_efb);
                let wire = bel.add_wire_rc_get(pin, format!("{pin}_PCNTR"));
                bel.ctx.claim_pip(wire, wire_efb);
            }
        }

        self.extract_bel(cell.bel(bslots::TSALL))
            .name("TSALL")
            .suffix("TSALL");

        let bcrd_sed = cell.bel(bslots::SED);
        self.extract_bel(bcrd_sed).name("SED").suffix("SED");

        let wire_osc = self.rc_wire(bcrd_osc.cell, "SEDSTDBY_OSC");
        self.add_bel_wire(bcrd_osc, "SEDSTDBY", wire_osc);
        let wire_osc_in = self.rc_wire(bcrd_osc.cell, "JSTDBY_OSC");
        self.claim_pip(wire_osc, wire_osc_in);
        let wire = self.rc_wire(cell, "SEDSTDBY_SED");
        self.add_bel_wire(bcrd_sed, "SEDSTDBY", wire);
        self.claim_pip(wire, wire_osc);

        if matches!(
            self.chip.kind,
            ChipKind::MachXo2(MachXo2Kind::MachXo3D | MachXo2Kind::MachNx)
        ) {
            let bcrd = cell.bel(bslots::ESB);
            let cell = cell.delta(1, 0);
            let mut bel = self.extract_bel_cell(bcrd, cell);
            bel.name_rc("ESB").suffix("ESB");
            let wire = bel.add_wire_rc_get("OSCCLK", "JOSCCLK_ESB");
            core::mem::drop(bel);

            let wire_osc = self.rc_wire(bcrd_osc.cell, "JOSCESB_OSC");
            self.add_bel_wire(bcrd_osc, "OSCESB", wire_osc);
            self.claim_pip(wire, wire_osc);
        }
    }

    fn process_config_ecp4(&mut self) {
        let cell = self.chip.special_loc[&SpecialLocKey::Config];

        self.extract_switchbox(cell.bel(bslots::BEL_INT))
            .output_bind_rc("PCNTRCLK", wires::IMUX_PCNTR_CLK.cell(0))
            .input_xlat_rc("JOSCCLK")
            .input_xlat_rc("JCIBCLK");

        let mut bel = self.extract_bel(cell.bel(bslots::JTAG));
        bel.names(["JTAG", "TCK", "TMS", "TDI", "TDO"])
            .suffix("JTAG");
        for pin in ["TCK", "TMS", "TDI", "TDO"] {
            let wire = bel.ctx.rc_wire(cell, &format!("J{pin}_JTAG"));
            let wire_pin = WireName {
                r: bel.ctx.chip.rows.len() as u8,
                c: 0,
                suffix: bel
                    .ctx
                    .naming
                    .strings
                    .get(&format!("J{pin}_{pin}"))
                    .unwrap(),
            };
            bel.add_wire(pin, wire);
            bel.add_wire(format!("{pin}_{pin}"), wire_pin);
            if pin == "TDO" {
                bel.ctx.claim_pip(wire_pin, wire);
            } else {
                bel.ctx.claim_pip(wire, wire_pin);
            }
        }
        core::mem::drop(bel);

        let bcrd_osc = cell.bel(bslots::OSC);
        self.extract_bel(bcrd_osc).name("OSC").suffix("OSC");

        self.extract_bel(cell.bel(bslots::START))
            .name("START")
            .suffix("START");

        self.extract_bel(cell.bel(bslots::GSR))
            .name("GSR")
            .suffix("GSR");

        let bcrd_efb = cell.bel(bslots::EFB);
        self.extract_bel(bcrd_efb)
            .names(["EFB", "CCLK"])
            .suffix("EFB");

        let cell_cclk = cell.with_col(self.chip.col_w());
        for (pin, pin_cclk) in [
            ("SPISCKEN", "PADDT"),
            ("SPISCKO", "PADDO"),
            ("SPISCKI", "PADDI"),
        ] {
            let wire = self.rc_io_wire(cell, &format!("J{pin}_EFB"));
            self.add_bel_wire(bcrd_efb, pin, wire);
            let wire_cclk = self.rc_io_wire(cell_cclk, &format!("J{pin_cclk}_CCLK"));
            self.add_bel_wire(bcrd_efb, format!("{pin}_CCLK"), wire_cclk);
            if pin_cclk == "PADDI" {
                self.claim_pip(wire, wire_cclk);
            } else {
                self.claim_pip(wire_cclk, wire);
            }
        }

        for (key, pin_i, pin_o, pin_oe) in [
            (SpecialIoKey::WriteN, "I2C1SCLI", "I2C1SCLO", "I2C1SCLOEN"),
            (SpecialIoKey::Cs1N, "I2C1SDAI", "I2C1SDAO", "I2C1SDAOEN"),
            (SpecialIoKey::D(0), "SPIMOSII", "SPIMOSIO", "SPIMOSIEN"),
            (SpecialIoKey::D(1), "SPIMISOI", "SPIMISOO", "SPIMISOEN"),
            (SpecialIoKey::Di, "", "SPIMCSN0", "SPICSNEN"),
        ] {
            let (cell_io, abcd) = self.xlat_io_loc_ecp4(self.chip.special_io[&key]);
            for (pin, wn_io) in [
                (pin_i, format!("JPADDI{abcd}_PIO")),
                (pin_o, format!("JPADDO{abcd}")),
                (pin_oe, format!("JPADDT{abcd}")),
            ] {
                if pin.is_empty() {
                    continue;
                }
                let wire = self.rc_io_wire(cell, &format!("J{pin}_EFB"));
                self.add_bel_wire(bcrd_efb, pin, wire);
                let wire_io = self.rc_io_wire(cell_io, &wn_io);
                if pin == pin_i {
                    self.claim_pip(wire, wire_io);
                } else {
                    self.claim_pip(wire_io, wire);
                }
            }
        }

        let cell_asb = cell.with_col(self.chip.col_w());
        for i in 0..8 {
            let wire = self.rc_io_wire(cell, &format!("JSCIDATI0_{i}_EFB"));
            self.add_bel_wire(bcrd_efb, format!("SCIDATI0_{i}"), wire);
            let wire_asb = self.rc_io_sn_wire(cell_asb, &format!("JSCIDATO{i}_ASB"));
            self.claim_pip(wire, wire_asb);
        }
        for (pin, pin_asb) in [
            ("SCIINT0", "SCIINT"),
            ("SCIRTYI0", "SCIRTYO"),
            ("SCIACKI0", "SCIACKO"),
        ] {
            let wire = self.rc_io_wire(cell, &format!("J{pin}_EFB"));
            self.add_bel_wire(bcrd_efb, pin, wire);
            let wire_asb = self.rc_io_sn_wire(cell_asb, &format!("J{pin_asb}_ASB"));
            self.claim_pip(wire, wire_asb);
        }
        for (pin, range) in [("SCISTBO", 0..64), ("SCIDATO", 0..8), ("SCIADRO", 0..12)] {
            for i in range {
                let wire = self.rc_io_wire(cell, &format!("J{pin}{i}_EFB"));
                self.add_bel_wire(bcrd_efb, format!("{pin}{i}"), wire);
            }
        }
        for pin in [
            "SCIWEO",
            "SCICYCO",
            "SCIRSTO",
            "SCICLKO",
            "SCISLEEP",
            "SCIINITEN0",
            "SCIINITEN1",
            "SCIRSTN",
        ] {
            let wire = self.rc_io_wire(cell, &format!("J{pin}_EFB"));
            self.add_bel_wire(bcrd_efb, pin, wire);
        }

        for (pin, range) in [
            ("SCIRTYI", 1..64),
            ("SCIACKI", 1..64),
            ("SCIINT", 1..64),
            ("SWSIMADDR", 0..32),
        ] {
            for i in range {
                let wire = self.rc_io_wire(cell, &format!("{pin}{i}_EFB"));
                self.add_bel_wire(bcrd_efb, format!("{pin}{i}"), wire);
            }
        }
        for i in 1..64 {
            for j in 0..8 {
                let wire = self.rc_io_wire(cell, &format!("SCIDATI{i}_{j}_EFB"));
                self.add_bel_wire(bcrd_efb, format!("SCIDATI{i}_{j}"), wire);
            }
        }

        {
            let mut bel = self.extract_bel(cell.bel(bslots::PCNTR));
            bel.name("PCNTR")
                .suffix("PCNTR")
                .input_xlat_rc(bcls::PCNTR::CLK, "CLK_PCNTR");

            for pin in ["CFGWAKE", "CFGSTDBY"] {
                let wire_efb = bel.ctx.rc_io_wire(cell, &format!("J{pin}_EFB"));
                bel.ctx.add_bel_wire(bcrd_efb, pin, wire_efb);
                let wire = bel.add_wire_rc_get(pin, format!("J{pin}_PCNTR"));
                bel.ctx.claim_pip(wire, wire_efb);
            }
        }

        let bcrd_sed = cell.bel(bslots::SED);
        self.extract_bel(bcrd_sed).name("SED").suffix("SED");

        let wire_osc = self.rc_wire(bcrd_osc.cell, "SEDSTDBY_OSC");
        self.add_bel_wire(bcrd_osc, "SEDSTDBY", wire_osc);
        let wire_osc_in = self.rc_wire(bcrd_osc.cell, "JSTDBY_OSC");
        self.claim_pip(wire_osc, wire_osc_in);
        let wire = self.rc_wire(cell, "SEDSTDBY_SED");
        self.add_bel_wire(bcrd_sed, "SEDSTDBY", wire);
        self.claim_pip(wire, wire_osc);
    }

    fn process_config_ecp5(&mut self) {
        let cell = self.chip.special_loc[&SpecialLocKey::Config];

        self.hybrid_out_rc(
            cell.bel(bslots::OSC),
            "JOSC_OSC",
            wires::OUT_OSC_OSC.cell(0),
        );
        self.hybrid_out_rc(
            cell.bel(bslots::SED),
            "JSEDCLKOUT_SED",
            wires::OUT_SED_SEDCLKOUT.cell(0),
        );
        self.hybrid_out_rc(
            cell.bel(bslots::JTAG),
            "JJTCK_JTAG",
            wires::OUT_JTAG_JTCK.cell(0),
        );

        let mut bel = self.extract_bel(cell.bel(bslots::JTAG));
        bel.names(["JTAG", "TCK", "TMS", "TDI", "TDO"])
            .suffix("JTAG");
        for pin in ["TCK", "TMS", "TDI", "TDO"] {
            let wire = bel.ctx.rc_wire(cell, &format!("J{pin}_JTAG"));
            let wire_pin = WireName {
                r: bel.ctx.chip.rows.len() as u8 + 1,
                c: 0,
                suffix: bel
                    .ctx
                    .naming
                    .strings
                    .get(&format!("J{pin}_{pin}"))
                    .unwrap(),
            };
            bel.add_wire(pin, wire);
            bel.add_wire(format!("{pin}_{pin}"), wire_pin);
            if pin == "TDO" {
                bel.ctx.claim_pip(wire_pin, wire);
            } else {
                bel.ctx.claim_pip(wire, wire_pin);
            }
        }
        core::mem::drop(bel);

        let bcrd_osc = cell.bel(bslots::OSC);
        self.extract_bel(bcrd_osc).name("OSC").suffix("OSC");

        self.extract_bel(cell.bel(bslots::START))
            .name("START")
            .suffix("START");

        self.extract_bel(cell.bel(bslots::GSR))
            .name("GSR")
            .suffix("GSR");

        self.extract_bel_cell(cell.bel(bslots::CCLK), cell.with_col(self.chip.col_w()))
            .name("CCLK")
            .suffix("CCLK");

        let bcrd_sed = cell.bel(bslots::SED);
        self.extract_bel(bcrd_sed).name("SED").suffix("SED");

        let wire_osc = self.rc_wire(cell, "SEDSTDBY_OSC");
        self.add_bel_wire(bcrd_osc, "SEDSTDBY", wire_osc);
        let wire_osc_in = self.rc_wire(cell, "JSTDBY_OSC");
        self.claim_pip(wire_osc, wire_osc_in);
        let wire = self.rc_wire(cell, "SEDSTDBY_SED");
        self.add_bel_wire(bcrd_sed, "SEDSTDBY", wire);
        self.claim_pip(wire, wire_osc);
    }

    fn process_config_crosslink(&mut self) {
        for tcid in [tcls::I2C_W, tcls::I2C_E] {
            for &tcrd in &self.edev.tile_index[tcid] {
                {
                    let mut bel = self.extract_bel(tcrd.bel(bslots::I2C));
                    bel.name(if tcid == tcls::I2C_W { "I2C1" } else { "I2C0" })
                        .suffix("I2C")
                        .add_wire_rc("PMUWKUP", "JPMUWKUP_I2C");
                    if tcid == tcls::I2C_E {
                        for (pin, key) in [("SDA", SpecialIoKey::D(2)), ("SCL", SpecialIoKey::D(3))]
                        {
                            let io = bel.ctx.chip.special_io[&key];
                            let (cell_io, abcd) = bel.ctx.xlat_io_loc_crosslink(io);

                            let wire_i =
                                bel.add_wire_rc_get(format!("{pin}I"), format!("J{pin}I_I2C"));
                            let wire_paddi =
                                bel.ctx.rc_io_wire(cell_io, &format!("JPADDI{abcd}_PIO"));
                            bel.ctx.claim_pip(wire_i, wire_paddi);

                            let wire_o =
                                bel.add_wire_rc_get(format!("{pin}O"), format!("J{pin}O_I2C"));
                            let wire_paddo = bel.ctx.rc_io_wire(cell_io, &format!("JPADDO{abcd}"));
                            bel.ctx.claim_pip(wire_paddo, wire_o);

                            let wire_oen =
                                bel.add_wire_rc_get(format!("{pin}OEN"), format!("J{pin}OEN_I2C"));
                            let wire_paddt = bel.ctx.rc_io_wire(cell_io, &format!("JPADDT{abcd}"));
                            bel.ctx.claim_pip(wire_paddt, wire_oen);
                        }
                    }
                }

                if tcid == tcls::I2C_E {
                    self.extract_bel_cell(tcrd.bel(bslots::NVCMTEST), tcrd.delta(1, 0))
                        .name("NVCMTEST")
                        .suffix("NVCMTEST");
                }
            }
        }

        {
            let cell = self.chip.special_loc[&SpecialLocKey::Osc];
            let bcrd = cell.bel(bslots::OSC);
            self.hybrid_out_rc(bcrd, "JLFCLKOUT_OSC", wires::OUT_OSC_LFCLKOUT.cell(0));
            self.hybrid_out_rc(bcrd, "JHFCLKOUT_OSC", wires::OUT_OSC_HFCLKOUT.cell(0));
            self.extract_bel(bcrd).name("OSC").suffix("OSC");
        }
        {
            let cell = self.chip.special_loc[&SpecialLocKey::Pmu];
            let mut bel = self.extract_bel(cell.bel(bslots::PMU));
            bel.names(["PMU", "PMUTEST"])
                .suffix("PMU")
                .suffix("PMUTEST");

            let wire = bel.add_wire_rc_get("PMUCLK", "JPMUCLK_PMU");
            let cell_osc = bel.ctx.chip.special_loc[&SpecialLocKey::Osc];
            let wire_osc = bel.ctx.rc_wire(cell_osc, "JLFCLKOUT_OSC");
            bel.ctx.claim_pip(wire, wire_osc);

            let wire = bel.add_wire_rc_get("PMUWKUP", "JPMUWKUP_PMU");
            let cell_i2c = cell.with_cr(bel.ctx.chip.col_e() - 1, bel.ctx.chip.row_n());
            let wire_i2c = bel.ctx.rc_wire(cell_i2c, "JPMUWKUP_I2C");
            bel.ctx.claim_pip(wire, wire_i2c);

            let wire = bel.add_wire_rc_get("USRWKUPN", "JUSRWKUPN_PMU");
            let io = bel.ctx.chip.special_io[&SpecialIoKey::PmuWakeupN];
            let (cell_io, abcd) = bel.ctx.xlat_io_loc_crosslink(io);
            let wire_paddi = bel.ctx.rc_io_wire(cell_io, &format!("JPADDI{abcd}_PIO"));
            bel.ctx.claim_pip(wire, wire_paddi);

            bel.add_wire_rc("SLEEPSTATUS", "SLEEPSTATUS_PMU");
        }
        {
            let cell = self.chip.special_loc[&SpecialLocKey::Config];

            self.extract_bel(cell.bel(bslots::GSR))
                .name("GSR")
                .suffix("GSR");

            self.extract_bel(cell.bel(bslots::CFGTEST))
                .name("CFGTEST")
                .suffix("CFGTEST");
        }
    }

    pub fn process_config(&mut self) {
        match self.chip.kind {
            ChipKind::Scm => {
                self.process_config_scm();
            }
            ChipKind::Ecp | ChipKind::Xp => {
                self.process_config_ecp();
            }
            ChipKind::MachXo => {
                self.process_config_machxo();
                self.process_osc_machxo();
            }
            ChipKind::Ecp2 | ChipKind::Ecp2M => {
                self.process_config_ecp2();
            }
            ChipKind::Xp2 => {
                self.process_config_xp2();
            }
            ChipKind::Ecp3(_) => {
                self.process_config_ecp3();
            }
            ChipKind::MachXo2(_) => {
                self.process_config_machxo2();
            }
            ChipKind::Ecp4 => {
                self.process_config_ecp4();
            }
            ChipKind::Ecp5 => {
                self.process_config_ecp5();
            }
            ChipKind::Crosslink => {
                self.process_config_crosslink();
            }
            ChipKind::Nexus(_) => {
                // TODO
            }
        }
    }
}
