use std::collections::{BTreeMap, BTreeSet, HashMap};

use prjcombine_interconnect::grid::{BelCoord, BelPadCoord};
use prjcombine_re_xilinx_rawdump::PkgPin;
use prjcombine_spartan6::{bond::Bond, defs::bcls};

use prjcombine_re_xilinx_naming_spartan6::ExpandedNamedDevice;
use prjcombine_re_xilinx_rd2db_grid::split_num;

pub fn make_bond(endev: &ExpandedNamedDevice, pins: &[PkgPin]) -> Bond {
    let mut bond_pins = BTreeMap::new();
    let mut io_banks = BTreeMap::new();
    let mut vref = BTreeSet::new();
    let io_lookup: HashMap<_, _> = endev
        .chip
        .get_bonded_ios()
        .into_iter()
        .map(|io| (endev.ngrid.get_bel_name(io.bel).unwrap(), io))
        .collect();
    let mut gt_lookup: HashMap<_, (String, BelPadCoord)> = HashMap::new();
    let mut gt_bank_lookup: HashMap<_, BelCoord> = HashMap::new();
    for gt in endev.get_gts() {
        let bank = gt.bank;
        gt_bank_lookup.insert(bank, gt.bel);
        for (i, &(pp, pn)) in gt.pads_clk.iter().enumerate() {
            gt_lookup.insert(
                pp,
                (
                    format!("MGTREFCLK{i}P_{bank}"),
                    gt.bel.pad(bcls::GTP::CLKP[i]),
                ),
            );
            gt_lookup.insert(
                pn,
                (
                    format!("MGTREFCLK{i}N_{bank}"),
                    gt.bel.pad(bcls::GTP::CLKN[i]),
                ),
            );
        }
        for (i, (pp, pn)) in gt.pads_rx.iter().enumerate() {
            gt_lookup.insert(
                pp,
                (format!("MGTRXP{i}_{bank}"), gt.bel.pad(bcls::GTP::RXP[i])),
            );
            gt_lookup.insert(
                pn,
                (format!("MGTRXN{i}_{bank}"), gt.bel.pad(bcls::GTP::RXN[i])),
            );
        }
        for (i, (pp, pn)) in gt.pads_tx.iter().enumerate() {
            gt_lookup.insert(
                pp,
                (format!("MGTTXP{i}_{bank}"), gt.bel.pad(bcls::GTP::TXP[i])),
            );
            gt_lookup.insert(
                pn,
                (format!("MGTTXN{i}_{bank}"), gt.bel.pad(bcls::GTP::TXN[i])),
            );
        }
    }
    let mut vcco_pins = vec![];
    for pin in pins {
        let bpin = if let Some(ref pad) = pin.pad {
            if let Some(&io) = io_lookup.get(&pad.as_str()) {
                //assert_eq!(pin.vref_bank, Some(bank));
                let bank = endev.chip.get_io_bank(io);
                let old = io_banks.insert(bank, pin.vcco_bank.unwrap());
                assert!(old.is_none() || old == Some(pin.vcco_bank.unwrap()));
                if pin.func.contains("VREF") {
                    vref.insert(io);
                }
                vec![io]
            } else if let Some(&(ref exp_func, pad)) = gt_lookup.get(&**pad) {
                if *exp_func != pin.func {
                    println!(
                        "pad {pad} got {f} exp {exp_func}",
                        f = pin.func,
                        pad = pad.to_string(endev.edev.db)
                    );
                }
                vec![pad]
            } else {
                println!("unk iopad {pad} {f}", f = pin.func);
                continue;
            }
        } else if let Some(b) = pin.func.strip_prefix("VCCO_")
            && let Ok(b) = b.parse()
        {
            let bank: u32 = b;
            vcco_pins.push((pin, bank));
            continue;
        } else {
            match &pin.func[..] {
                "NC" => vec![],
                "GND" => vec![endev.chip.pad_gnd()],
                "VCCINT" => vec![endev.chip.pad_vccint()],
                "VCCAUX" => vec![endev.chip.pad_vccaux()],
                "VBATT" => vec![endev.chip.pad_vccbatt()],
                "VFS" => vec![endev.chip.pad_vfs()],
                "RFUSE" => vec![endev.chip.pad_rfuse()],
                "TCK" => vec![endev.chip.pad_tck()],
                "TDI" => vec![endev.chip.pad_tdi()],
                "TDO" => vec![endev.chip.pad_tdo()],
                "TMS" => vec![endev.chip.pad_tms()],
                "CMPCS_B_2" => vec![endev.chip.pad_tspi_cs_b()],
                "DONE_2" => vec![endev.chip.pad_done()],
                "PROGRAM_B_2" => vec![endev.chip.pad_prog_b()],
                "SUSPEND" => vec![endev.chip.pad_suspend()],
                _ => {
                    if let Some((n, b)) = split_num(&pin.func) {
                        let bel = gt_bank_lookup[&b];
                        let pad = match n {
                            "MGTAVCC_" => bcls::GTP::AVCC,
                            "MGTAVCCPLL0_" => bcls::GTP::AVCCPLL[0],
                            "MGTAVCCPLL1_" => bcls::GTP::AVCCPLL[1],
                            "MGTAVTTRX_" => bcls::GTP::AVTTRX,
                            "MGTAVTTTX_" => bcls::GTP::AVTTTX,
                            "MGTRREF_" => bcls::GTP::RREF,
                            "MGTAVTTRCAL_" => bcls::GTP::AVTTRCAL,
                            _ => {
                                println!("UNK FUNC {}", pin.func);
                                continue;
                            }
                        };
                        vec![bel.pad(pad)]
                    } else {
                        println!("UNK FUNC {}", pin.func);
                        continue;
                    }
                }
            }
        };
        bond_pins.insert(pin.pin.clone(), bpin);
    }
    let mut bank_to_pads: BTreeMap<u32, Vec<_>> = BTreeMap::new();
    for (real_bank, bank) in io_banks {
        bank_to_pads
            .entry(bank)
            .or_default()
            .push(endev.chip.pad_bank_vcco(real_bank));
    }
    for (pin, bank) in vcco_pins {
        bond_pins.insert(pin.pin.clone(), bank_to_pads[&bank].clone());
    }
    Bond {
        pins: bond_pins,
        vref,
    }
}
