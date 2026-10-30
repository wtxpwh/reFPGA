use clap::Parser;
use ndarray::Array2;
use prjcombine_entity::{EntityId, EntitySet, EntityVec};
use prjcombine_re_lattice_dump::{parse_name, parse_tiles};
use prjcombine_re_lattice_rawdump::{Db, Grid, Part, Pip};
use prjcombine_re_lattice_rawdump::{GridId, Tile};
use prjcombine_re_toolchain::Toolchain;
use rayon::ThreadPoolBuilder;
use rayon::prelude::*;
use regex::regex;
use std::collections::BTreeMap;
use std::error::Error;
use std::fs::File;
use std::fs::create_dir_all;
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::Stdio;

struct FamilyInfo {
    name: &'static str,
    parts: &'static [(&'static str, &'static str, &'static str)],
    pkgs: &'static [&'static str],
    speeds: &'static [&'static str],
}

const RADIANT_FAMILIES: &[FamilyInfo] = &[
    FamilyInfo {
        name: "machxo2",
        parts: &[
            ("machxo4", "lfmxo4", "LFMXO4-010HE"),
            ("machxo4", "lfmxo4", "LFMXO4-010HC"),
            ("machxo4", "lfmxo4", "LFMXO4-010ZC"),
            ("machxo4", "lfmxo4", "LFMXO4-015HE"),
            ("machxo4", "lfmxo4", "LFMXO4-015HC"),
            ("machxo4", "lfmxo4", "LFMXO4-015ZC"),
            ("machxo4", "lfmxo4", "LFMXO4-025HE"),
            ("machxo4", "lfmxo4", "LFMXO4-025HC"),
            ("machxo4", "lfmxo4", "LFMXO4-025ZC"),
            ("machxo4", "lfmxo4", "LFMXO4-050HE"),
            ("machxo4", "lfmxo4", "LFMXO4-050HC"),
            ("machxo4", "lfmxo4", "LFMXO4-050ZC"),
            ("machxo4", "lfmxo4", "LFMXO4-080HE"),
            ("machxo4", "lfmxo4", "LFMXO4-080HC"),
            ("machxo4", "lfmxo4", "LFMXO4-080ZC"),
            ("machxo4", "lfmxo4", "LFMXO4-110HE"),
            ("machxo4", "lfmxo4", "LFMXO4-110HC"),
            ("machxo4d", "lfmxo4", "LFMXO4D-040HC"),
            ("machxo4d", "lfmxo4", "LFMXO4D-040ZC"),
            ("machxo4d", "lfmxo4", "LFMXO4D-050HC"),
            ("machxo4d", "lfmxo4", "LFMXO4D-050ZC"),
            ("machxo4d", "lfmxo4", "LFMXO4D-080HE"),
            ("machxo4d", "lfmxo4", "LFMXO4D-080HC"),
            ("machxo4d", "lfmxo4", "LFMXO4D-080ZC"),
            ("machxo4d", "lfmxo4", "LFMXO4D-110HE"),
            ("machxo4d", "lfmxo4", "LFMXO4D-110HC"),
            ("machxo4d", "lfmxo4", "LFMXO4D-110ZC"),
        ],
        pkgs: &[
            "UUG36", "UUG49", "UUG81", "UTG69", "TSG100", "TSG144", "BSG132", "BBG256", "BFG256",
            "BBG400", "BBG484", "BTG256", "BTG400",
        ],
        speeds: &["2", "3", "5", "6"],
    },
    FamilyInfo {
        name: "nexus",
        parts: &[
            ("je5d15", "lifcl", "LIFCL-17"),
            ("je5d15", "lfd2nx", "LFD2NX-9"),
            ("je5d15", "lfd2nx", "LFD2NX-17"),
            ("je5d30", "lifcl", "LIFCL-40"),
            ("je5d30", "lfd2nx", "LFD2NX-28"),
            ("je5d30", "lfd2nx", "LFD2NX-40"),
            ("je5d30", "ut24c", "UT24C40"),
            ("jd5d80", "lfcpnx", "LFCPNX-50"),
            ("jd5d80", "lfcpnx", "LFCPNX-50P"),
            ("jd5d80", "lfcpnx", "LFCPNX-100"),
            ("jd5d80", "lfcpnx", "LFCPNX-100P"),
            ("jd5d80", "lfcpnx", "LFCPNX-100AUTODIE"),
            ("jd5d80", "ut24cp", "UT24CP100"),
            ("jd5d80", "lfmxo5", "LFMXO5-55T"),
            ("jd5d80", "lfmxo5", "LFMXO5-100T"),
            ("jd5f25", "lfd2nx", "LFD2NX-15"),
            ("jd5f25", "lfd2nx", "LFD2NX-25"),
            ("jd5f25", "lfd2nx", "LFD2NX-15P"),
            ("jd5f25", "lfd2nx", "LFD2NX-25P"),
            ("jd5f25", "lfmxo5", "LFMXO5-25"),
            ("jd5f25", "lfmxo5", "LFMXO5-25P"),
            ("jd5u27", "lifcl", "LIFCL-33"),
            ("jd5u27a", "lifcl", "LIFCL-33U"),
            ("jd5t55", "lfd2nx", "LFD2NX-35"),
            ("jd5t55", "lfd2nx", "LFD2NX-65"),
            ("jd5t55", "lfmxo5", "LFMXO5-35"),
            ("jd5t55", "lfmxo5", "LFMXO5-35T"),
            ("jd5t55", "lfmxo5", "LFMXO5-65"),
            ("jd5t55", "lfmxo5", "LFMXO5-65T"),
        ],
        pkgs: &[
            // LFD2NX, LIFCL, UT24C
            "QFN72",
            "WLCSP72",
            "WLCSP84",
            "FCCSP104",
            "CSFBGA121",
            "CSFBGA196",
            "CABGA196",
            "CABGA256",
            "CABGA400",
            "CABGA484",
            "CSBGA289",
            // LFCPNX, UT24CP
            "ASG256",
            "CBG256",
            "BBG484",
            "BFG484",
            "LFG672",
            "LFG1156",
            // LFMXO5
            "BBG256",
            "BBG400",
        ],
        speeds: &[
            "7_High-Performance_1.0V",
            "7_Low-Power_1.0V",
            "8_High-Performance_1.0V",
            "8_Low-Power_1.0V",
            "9_High-Performance_1.0V",
            "9_Low-Power_1.0V",
        ],
    },
    FamilyInfo {
        name: "avant",
        parts: &[
            ("ap6a160", "ln2-ct", "LN2-CT-20ES"),
            ("ap6a160", "ln2-ct", "LN2-CT-20ES1"),
            ("ap6a160", "ln2-ct", "LN2-CT-20"),
            ("ap6a160", "ln2-ct", "LN2-CT-20E"),
            ("ap6a160", "ln2-ct", "LN2-CT-16"),
            ("ap6a160", "ln2-ct", "LN2-CT-16E"),
            ("ap6a160", "ln2-mh", "LN2-MH-20ES1"),
            ("ap6a160", "ln2-mh", "LN2-MH-20"),
            ("ap6a160", "ln2-mh", "LN2-MH-70ES1"),
            ("ap6a160", "lav-at", "LAV-AT-E30"),
            ("ap6a160", "lav-at", "LAV-AT-E30B"),
            ("ap6a160", "lav-at", "LAV-AT-G30"),
            ("ap6a160", "lav-at", "LAV-AT-X30"),
            ("ap6a160", "lav-at", "LAV-AT-E30L"),
            ("ap6a160", "lav-at", "LAV-AT-G30L"),
            ("ap6a160", "lav-at", "LAV-AT-X30L"),
            ("ap6a400", "lav-at", "LAV-AT-E50"),
            ("ap6a400", "lav-at", "LAV-AT-E70"),
            ("ap6a400", "lav-at", "LAV-AT-E70B"),
            ("ap6a400", "lav-at", "LAV-AT-E70ES1"),
            ("ap6a400", "lav-at", "LAV-AT-G50"),
            ("ap6a400", "lav-at", "LAV-AT-G70"),
            ("ap6a400", "lav-at", "LAV-AT-G70ES"),
            ("ap6a400", "lav-at", "LAV-AT-X50"),
            ("ap6a400", "lav-at", "LAV-AT-X70"),
            ("ap6a400", "lav-at", "LAV-AT-X70ES"),
            ("ap6a400", "lav-at", "LAV-AT-E50L"),
            ("ap6a400", "lav-at", "LAV-AT-G50L"),
            ("ap6a400", "lav-at", "LAV-AT-X50L"),
            ("ap6a400", "lav-at", "LAV-AT-E70L"),
            ("ap6a400", "lav-at", "LAV-AT-G70L"),
            ("ap6a400", "lav-at", "LAV-AT-X70L"),
        ],
        pkgs: &[
            "ASG410",
            "ASGA410",
            "CBG324",
            "CBG484",
            "CSFBGA285",
            "CSG841",
            "FBG644",
            "FBGA484",
            "LBG484",
            "LFG676",
            "LFG1156",
        ],
        speeds: &["1", "2", "3"],
    },
];

#[derive(Debug, Parser)]
#[command(
    name = "dump_radiant_parts",
    about = "Dump Radiant part geometry into rawdump files."
)]
struct Args {
    toolchain: PathBuf,
    target_directory: PathBuf,
    families: Vec<String>,
    #[arg(short, long, default_value = "0")]
    num_threads: usize,
}

fn dump_tiles(
    tc: &Toolchain,
    family: &str,
    arch: &str,
    dev: &str,
    pkg: &str,
    spd: &str,
) -> Option<Array2<Tile>> {
    let mut dir = tempfile::Builder::new()
        .prefix("prjcombine_radiant_dump")
        .tempdir()
        .unwrap();
    let mut file = File::create(dir.path().join("top.v")).unwrap();
    writeln!(
        file,
        r#"
(* \db:architecture ="{arch}", \db:device ="{dev}", \db:package ="{pkg}", \db:speed ="{spd}", \db:view ="physical" *)
module top();
(*keep*)
meow meow();
endmodule
module meow();
endmodule;
"#
    ).unwrap();
    std::mem::drop(file);
    let mut cmd = tc.command("sv2udb").unwrap();
    cmd.current_dir(dir.path().as_os_str());
    cmd.stdin(Stdio::null());
    cmd.arg("-o");
    cmd.arg("top.udb");
    cmd.arg("top.v");
    let status = cmd.output().unwrap();
    if !status.status.success() {
        println!("non-zero sv2udb exit status");
        let _ = std::io::stderr().write_all(&status.stdout);
        let _ = std::io::stderr().write_all(&status.stderr);
        return None;
    }
    println!("dumping {dev}-{pkg} tiles...");
    let mut cmd = tc.command("bitgen").unwrap();
    cmd.current_dir(dir.path().as_os_str());
    cmd.stdin(Stdio::null());
    cmd.arg("-b");
    if arch != "lfmxo4" {
        cmd.arg("-ipeval");
    }
    cmd.arg("top.udb");
    let status = cmd.output().unwrap();
    if !status.status.success() {
        let _ = std::io::stderr().write_all(&status.stdout);
        let _ = std::io::stderr().write_all(&status.stderr);
        dir.disable_cleanup(true);
        println!("non-zero bitgen exit status on {dev}-{pkg}");
    }

    let Ok(rbt) = File::open(dir.path().join("top.rbt")) else {
        println!("no bitstream on {dev}-{pkg}");
        return None;
    };
    let mut orbt = File::create(dir.path().join("top2.rbt")).unwrap();
    for line in BufReader::new(rbt).lines() {
        let line = line.unwrap();
        if !line.starts_with(['0', '1']) {
            writeln!(orbt, "{line}").unwrap();
        }
    }
    std::mem::drop(orbt);

    let mut cmd = tc.command("bstool").unwrap();
    cmd.current_dir(dir.path().as_os_str());
    cmd.stdin(Stdio::null());
    cmd.arg("-t");
    cmd.arg("top2.rbt");
    let status = cmd.output().unwrap();
    if !status.status.success() {
        println!("non-zero bstool exit status on {dev}-{pkg}");
        let _ = std::io::stderr().write_all(&status.stdout);
        let _ = std::io::stderr().write_all(&status.stderr);
        return None;
    }
    Some(parse_tiles(
        std::str::from_utf8(&status.stdout).unwrap(),
        family,
    ))
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ProbeResult {
    Error,
    NoArch,
    NoDev,
    NoPkg,
    NoSpd(String),
    Found(String),
}

fn probe_device(tc: &Toolchain, arch: &str, dev: &str, pkg: &str, spd: &str) -> ProbeResult {
    let dir = tempfile::Builder::new()
        .prefix("prjcombine_radiant_dump__probe")
        .tempdir()
        .unwrap();
    let mut cmd = match tc.command("nodefinder") {
        Ok(c) => c,
        Err(err) => {
            println!("PROBE FAILED {arch} {dev} {pkg} {spd}: {err:?}");
            return ProbeResult::Error;
        }
    };
    cmd.current_dir(dir.path())
        .arg(arch)
        .arg(dev)
        .arg(pkg)
        .arg(spd)
        .arg("nya");
    let status = match cmd.output() {
        Ok(s) => s,
        Err(err) => {
            println!("PROBE FAILED {arch} {dev} {pkg} {spd}: {err:?}");
            return ProbeResult::Error;
        }
    };
    let stderr = std::str::from_utf8(&status.stderr).unwrap();
    if stderr.contains("ERROR - No personality module specification") {
        return ProbeResult::NoArch;
    } else if stderr.contains("ERROR - No Device specification") {
        return ProbeResult::NoDev;
    } else if stderr.contains("ERROR - No Pkg specification") {
        return ProbeResult::NoPkg;
    } else if !status.status.success() {
        println!("NON-ZERO PROBE STATUS {arch} {dev} {pkg} {spd}:");
        let _ = std::io::stderr().write_all(&status.stdout);
        let _ = std::io::stderr().write_all(&status.stderr);
        return ProbeResult::Error;
    }
    let stdout = std::str::from_utf8(&status.stdout).unwrap();
    let Some(line) = stdout.lines().next() else {
        println!("MISSING DEVICE LINE {arch} {dev} {pkg} {spd}");
        return ProbeResult::Error;
    };
    let dev_re = regex!(
        r"Loading device for application GENERIC from file '([a-zA-Z0-9]+).nph' in environment"
    );
    let Some(cap) = dev_re.captures(line) else {
        println!("WEIRD DEVICE LINE {arch} {dev} {pkg} {spd}: {line}");
        return ProbeResult::Error;
    };
    let name = cap[1].to_string();
    if stderr.contains("ERROR - No Speed specification") {
        ProbeResult::NoSpd(name)
    } else {
        ProbeResult::Found(name)
    }
}

struct PartInfo {
    arch: &'static str,
    dev: &'static str,
    pkg: &'static str,
    speed: Vec<&'static str>,
}

struct GridInfo {
    bucket: &'static str,
    nph: String,
    parts: Vec<PartInfo>,
}

fn probe_devices(tc: &Toolchain, family: &FamilyInfo) -> Vec<GridInfo> {
    let parts: Vec<_> = family
        .parts
        .par_iter()
        .copied()
        .filter(
            |&(_bucket, arch, dev)| match probe_device(tc, arch, dev, "nya", "nya") {
                ProbeResult::Error => false,
                ProbeResult::NoArch => {
                    println!("MISSING ARCH {arch}");
                    false
                }
                ProbeResult::NoDev => {
                    println!("MISSING DEV {arch} {dev}");
                    false
                }
                ProbeResult::NoPkg | ProbeResult::NoSpd(_) | ProbeResult::Found(_) => {
                    println!("PROBED DEV {arch} {dev}");
                    true
                }
            },
        )
        .flat_map(|(bucket, arch, dev)| {
            Vec::from_iter(family.pkgs.iter().map(|&pkg| (bucket, arch, dev, pkg)))
        })
        .filter_map(
            |(bucket, arch, dev, pkg)| match probe_device(tc, arch, dev, pkg, "nya") {
                ProbeResult::Error => None,
                ProbeResult::NoArch => {
                    println!("MISSING ARCH {arch} IN STAGE 2?!?");
                    None
                }
                ProbeResult::NoDev => {
                    println!("MISSING DEV {arch} {dev} IN STAGE 2?!?");
                    None
                }
                ProbeResult::NoPkg => None,
                ProbeResult::NoSpd(name) | ProbeResult::Found(name) => {
                    let speeds: Vec<_> = family
                        .speeds
                        .par_iter()
                        .copied()
                        .filter(|spd| {
                            matches!(probe_device(tc, arch, dev, pkg, spd), ProbeResult::Found(_))
                        })
                        .collect();
                    println!("PROBED PKG {arch} {dev} {pkg}: {name} {speeds:?}");
                    Some((bucket, arch, dev, pkg, name, speeds))
                }
            },
        )
        .collect();
    let mut grids = BTreeMap::new();
    for (bucket, arch, dev, pkg, nph, speed) in parts {
        grids.entry((bucket, nph)).or_insert(vec![]).push(PartInfo {
            arch,
            dev,
            pkg,
            speed,
        });
    }
    grids
        .into_iter()
        .map(|((bucket, nph), parts)| GridInfo { bucket, nph, parts })
        .collect()
}

fn dump_nodes(tc: &Toolchain, arch: &str, dev: &str, pkg: &str, spd: &str, nph: String) -> Grid {
    let node_re = regex!(r"^Found node ([a-zA-Z0-9_]+)$");
    let pip_re =
        regex!(r"^Found connection from ([a-zA-Z0-9_]+) to ([a-zA-Z0-9_]+) (\(pseudo\))?$");

    let mut strings = EntitySet::new();
    let mut nodes = EntitySet::new();
    let mut pips = BTreeMap::new();

    let dir = tempfile::Builder::new()
        .prefix("prjcombine_radiant_dump__nodes")
        .tempdir()
        .unwrap();
    let mut cmd = tc.command("nodefinder_improved").unwrap();
    cmd.current_dir(dir.path())
        .arg(arch)
        .arg(dev)
        .arg(pkg)
        .arg(spd)
        .arg("nya")
        .arg("nya");
    let status = cmd.output().unwrap();
    if !status.status.success() {
        println!("WHOOPS FAILED TO GET NODES");
        let _ = std::io::stderr().write_all(&status.stdout);
        let _ = std::io::stderr().write_all(&status.stderr);
        panic!("uhoh");
    }
    let stdout = std::str::from_utf8(&status.stdout).unwrap();
    for line in stdout.lines() {
        if let Some(cap) = node_re.captures(line) {
            nodes.insert(parse_name(&mut strings, &cap[1]));
        } else if let Some(cap) = pip_re.captures(line) {
            let nf = nodes.insert(parse_name(&mut strings, &cap[1])).0;
            let nt = nodes.insert(parse_name(&mut strings, &cap[2])).0;
            let is_pseudo = cap.get(3).is_some();
            pips.insert((nf, nt), Pip { is_pseudo });
        } else if line.starts_with("Found") {
            panic!("FAILED TO PARSE: {line}");
        }
    }

    Grid {
        nph,
        tiles: Array2::from_shape_simple_fn((0, 0), || unreachable!()),
        strings,
        nodes,
        pips,
    }
}

fn dump_grid(
    tc: &Toolchain,
    family: &FamilyInfo,
    pgrid: GridInfo,
) -> (&'static str, Grid, Vec<Part>) {
    let part = if family.name == "machxo2" {
        pgrid.parts.last().unwrap()
    } else {
        pgrid.parts.first().unwrap()
    };
    let mut grid = dump_nodes(tc, part.arch, part.dev, part.pkg, part.speed[0], pgrid.nph);
    println!("dumped {} nodes", grid.nph);
    if family.name != "avant" {
        grid.tiles = dump_tiles(
            tc,
            family.name,
            part.arch,
            part.dev,
            part.pkg,
            part.speed[0],
        )
        .unwrap();
        println!("dumped {} tiles", grid.nph);
    }
    let mut parts = vec![];
    for part in pgrid.parts {
        parts.push(Part {
            arch: part.arch.to_string(),
            name: part.dev.to_string(),
            package: part.pkg.to_string(),
            speeds: part.speed.into_iter().map(|s| s.to_string()).collect(),
            // fixed up later
            grid: GridId::from_idx(0),
            // uhhhhh. how do we get this.
            sites: vec![],
        });
    }
    (pgrid.bucket, grid, parts)
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    ThreadPoolBuilder::new()
        .num_threads(args.num_threads)
        .build_global()
        .unwrap();
    let tc = Toolchain::from_file(&args.toolchain)?;
    create_dir_all(&args.target_directory)?;
    for family in RADIANT_FAMILIES {
        if args.families.iter().any(|x| x == family.name) {
            let pre_grids = probe_devices(&tc, family);
            for grid in &pre_grids {
                println!("GRID {bucket} {nph}:", bucket = grid.bucket, nph = grid.nph);
                for part in &grid.parts {
                    println!(
                        "    PART {arch} {dev} {pkg} {spd:?}",
                        arch = part.arch,
                        dev = part.dev,
                        pkg = part.pkg,
                        spd = part.speed
                    )
                }
            }
            let grid_parts: Vec<_> = pre_grids
                .into_par_iter()
                .map(|pgrid| dump_grid(&tc, family, pgrid))
                .collect();
            let mut buckets = BTreeMap::new();
            for (bname, grid, pparts) in grid_parts {
                let bucket = buckets.entry(bname).or_insert_with(|| Db {
                    family: family.name.to_string(),
                    grids: EntityVec::new(),
                    parts: vec![],
                });
                let gid = bucket.grids.push(grid);
                for mut part in pparts {
                    part.grid = gid;
                    bucket.parts.push(part);
                }
            }
            for (name, db) in buckets {
                db.to_file(args.target_directory.join(format!("{name}.zstd")))
                    .unwrap();
                println!("dumped {name} [{n} grids]", n = db.grids.len());
            }
        }
    }
    Ok(())
}
