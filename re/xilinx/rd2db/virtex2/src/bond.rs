use std::collections::{BTreeMap, BTreeSet, HashMap};

use prjcombine_interconnect::dir::DirV;
use prjcombine_re_xilinx_rawdump::PkgPin;
use prjcombine_types::Polarity;
use prjcombine_virtex2::bond::Bond;

use prjcombine_re_xilinx_naming_virtex2::ExpandedNamedDevice;
use prjcombine_re_xilinx_rd2db_grid::split_num;
use prjcombine_virtex2::chip::ChipKind;

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
    let gt_lookup: HashMap<_, _> = endev
        .chip
        .cols_gt
        .iter()
        .flat_map(|(&col, &(bs, bn))| {
            [
                (bs, endev.chip.bel_gt(DirV::S, col)),
                (bn, endev.chip.bel_gt(DirV::N, col)),
            ]
        })
        .collect();
    let mut vcco_pins = vec![];
    for pin in pins {
        let bpin = if let Some(ref pad) = pin.pad {
            if pad.starts_with("PAD") || pad.starts_with("IPAD") || pad.starts_with("CLK") {
                let io = io_lookup[&pad.as_str()];
                let info = endev.chip.get_io_info(io);
                if endev.chip.kind != ChipKind::FpgaCore {
                    assert_eq!(pin.vref_bank, Some(info.bank));
                    let old = io_banks.insert(info.bank, pin.vcco_bank.unwrap());
                    assert!(old.is_none() || old == Some(pin.vcco_bank.unwrap()));
                    if pin.func.contains("VREF_") {
                        vref.insert(io);
                    }
                } else {
                    assert_eq!(pin.vref_bank, None);
                    assert_eq!(pin.vcco_bank, None);
                }
                vec![io]
            } else if let Some((n, b)) = split_num(pad) {
                let bel = gt_lookup[&b];
                let pad = match n {
                    "RXPPAD" => endev.chip.pad_gt_rx(bel, Polarity::P),
                    "RXNPAD" => endev.chip.pad_gt_rx(bel, Polarity::N),
                    "TXPPAD" => endev.chip.pad_gt_tx(bel, Polarity::P),
                    "TXNPAD" => endev.chip.pad_gt_tx(bel, Polarity::N),
                    _ => panic!("FUNNY PAD {pad}"),
                };
                vec![pad]
            } else {
                panic!("FUNNY PAD {pad}");
            }
        } else if pin.func.starts_with("VCCO_") || pin.func == "VCCO" {
            vcco_pins.push((pin, pin.vcco_bank.unwrap()));
            continue;
        } else {
            match &pin.func[..] {
                "NC" => vec![],
                "RSVD" => vec![
                    endev.chip.pad_dx(Polarity::P),
                    endev.chip.pad_dx(Polarity::N),
                ], // virtex2: likely DXP/DXN
                "GND" => vec![endev.chip.pad_gnd()],
                "VCCINT" => vec![endev.chip.pad_vccint()],
                "VCCAUX" => vec![endev.chip.pad_vccaux()],
                "VBATT" => vec![endev.chip.pad_vccbatt()],
                "TCK" => vec![endev.chip.pad_tck()],
                "TDI" => vec![endev.chip.pad_tdi()],
                "TDO" => vec![endev.chip.pad_tdo()],
                "TMS" => vec![endev.chip.pad_tms()],
                "CCLK" => vec![endev.chip.pad_cclk()],
                "DONE" => vec![endev.chip.pad_done()],
                "PROG_B" => vec![endev.chip.pad_prog_b()],
                "M0" => vec![endev.chip.pads_mode()[0]],
                "M1" => vec![endev.chip.pads_mode()[1]],
                "M2" => vec![endev.chip.pads_mode()[2]],
                "HSWAP_EN" => vec![endev.chip.pad_hswap_en()],
                "PWRDWN_B" => vec![endev.chip.pad_powerdown_b()],
                "SUSPEND" => vec![endev.chip.pad_suspend()],
                "DXN" => vec![endev.chip.pad_dx(Polarity::N)],
                "DXP" => vec![endev.chip.pad_dx(Polarity::P)],
                _ => {
                    if let Some((n, b)) = split_num(&pin.func) {
                        let bel = gt_lookup[&b];
                        let pad = match n {
                            "GNDA" => endev.chip.pad_gt_gnda(bel),
                            "VTRXPAD" => endev.chip.pad_gt_vtrx(bel),
                            "VTTXPAD" => endev.chip.pad_gt_vttx(bel),
                            "AVCCAUXRX" => endev.chip.pad_gt_avccauxrx(bel),
                            "AVCCAUXTX" => endev.chip.pad_gt_avccauxtx(bel),
                            _ => {
                                println!("UNK FUNC {}", pin.func);
                                continue;
                            }
                        };
                        vec![pad]
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
            .push(endev.chip.pad_bank_vcco(real_bank).unwrap());
    }
    for (pin, bank) in vcco_pins {
        bond_pins.insert(pin.pin.clone(), bank_to_pads[&bank].clone());
    }
    Bond {
        pins: bond_pins,
        vref,
    }
}
