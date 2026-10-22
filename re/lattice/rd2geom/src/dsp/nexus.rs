use prjcombine_ecp::defs::{bcls, bslots, tcls};

use crate::ChipContext;

impl ChipContext<'_> {
    pub(super) fn process_dsp_nexus(&mut self) {
        for tcid in [tcls::DSP_W, tcls::DSP_E] {
            for &tcrd in &self.edev.tile_index[tcid] {
                for idx in 0..2 {
                    let bcrd = tcrd.bel(bslots::DSP[idx]);
                    let cell = tcrd.delta(idx as i32 * 4, 0);
                    let (r, c) = self.rc(tcrd.cell);
                    let c = c + (idx as u8) * 4;
                    let c1 = c + 1;
                    let c2 = c + 2;
                    let c3 = c + 3;
                    let mut names = vec![
                        format!("PREADD9_CORE_R{r}C{c1}A"),
                        format!("PREADD9_CORE_R{r}C{c1}B"),
                        format!("PREADD9_CORE_R{r}C{c2}A"),
                        format!("PREADD9_CORE_R{r}C{c2}B"),
                        format!("MULT9_CORE_R{r}C{c1}A"),
                        format!("MULT9_CORE_R{r}C{c1}B"),
                        format!("MULT9_CORE_R{r}C{c2}A"),
                        format!("MULT9_CORE_R{r}C{c2}B"),
                        format!("MULT18_CORE_R{r}C{c1}"),
                        format!("MULT18_CORE_R{r}C{c2}"),
                        format!("MULT18X36_CORE_R{r}C{c3}"),
                        format!("REG18_CORE_R{r}C{c3}A"),
                        format!("REG18_CORE_R{r}C{c3}B"),
                        format!("REG18_CORE_R{r}C{c3}C"),
                        format!("REG18_CORE_R{r}C{c3}D"),
                        format!("ACC54_CORE_R{r}C{c3}"),
                    ];
                    if idx == 1 {
                        names.push(format!("MULT36_CORE_R{r}C{c3}"));
                    }
                    let mut prev9_2 = None;
                    let mut prev9 = None;
                    let mut prev_acc = None;
                    if idx == 1
                        || self
                            .edev
                            .bel_delta(bcrd.cell, -12, 0, bslots::DSP[0])
                            .is_some()
                    {
                        let prev_cell = if idx == 1 {
                            cell.delta(-2, 0)
                        } else {
                            cell.delta(-6, 0)
                        };
                        let prev_idx = [3, 1][idx];
                        prev9 = Some((prev_cell, prev_idx, 'H'));
                        prev9_2 = Some((prev_cell, prev_idx, 'L'));
                        prev_acc = Some(prev_cell.delta(1, 0));
                    }
                    let mut bel = self.extract_bel(bcrd);
                    bel.names(names);
                    for i in 0..2 {
                        let cur_cell = cell.delta(i as i32 + 1, 0);
                        bel.cell = cur_cell;
                        let ii = idx * 2 + i;
                        for (j, hl) in ['L', 'H'].into_iter().enumerate() {
                            for k in 0..9 {
                                bel.input_xlat_rc(
                                    [bcls::DSP::A0, bcls::DSP::A1][i][j * 9 + k],
                                    format!("JA{k}_MULT9_CORE_MULT9_{hl}{ii}"),
                                )
                                .input_xlat_rc(
                                    [bcls::DSP::B0, bcls::DSP::B1][i][j * 9 + k],
                                    format!("JB{k}_PREADD9_CORE_PREADD9_{hl}{ii}"),
                                );
                                for pname in ["BLSO", "BRSO", "BLS1", "BLS2", "BRS1", "BRS2", "BR"]
                                {
                                    bel.add_wire_rc(
                                        format!("{pname}{k}_PREADD9_{hl}{i}"),
                                        format!("J{pname}{k}_PREADD9_CORE_PREADD9_{hl}{ii}"),
                                    );
                                }
                                for pname in ["BR", "AS1", "AS2", "AR", "AO", "BO"] {
                                    bel.add_wire_rc(
                                        format!("{pname}{k}_MULT9_{hl}{i}"),
                                        format!("J{pname}{k}_MULT9_CORE_MULT9_{hl}{ii}"),
                                    );
                                }
                                for ab in ['A', 'B'] {
                                    bel.add_wire_rc(
                                        format!("{ab}R{hl}{k}_MULT18_{i}"),
                                        format!("J{ab}R{hl}{k}_MULT18_CORE_MULT18_{ii}"),
                                    )
                                    .claim_pip(
                                        format!("{ab}R{hl}{k}_MULT18_{i}"),
                                        format!("{ab}O{k}_MULT9_{hl}{i}"),
                                    );
                                }
                                bel.claim_pip(
                                    format!("BR{k}_MULT9_{hl}{i}"),
                                    format!("BR{k}_PREADD9_{hl}{i}"),
                                );
                                for (d, prev) in [(1, prev9), (2, prev9_2)] {
                                    if let Some((prev_cell, prev_idx, prev_hl)) = prev {
                                        let wire_prev = bel.ctx.rc_wire(
                                            prev_cell,
                                            &format!(
                                                "JBRSO{k}_PREADD9_CORE_PREADD9_{prev_hl}{prev_idx}"
                                            ),
                                        );
                                        bel.claim_pip_from(
                                            format!("BRS{d}{k}_PREADD9_{hl}{i}"),
                                            wire_prev,
                                        );
                                        let wire_prev = bel.ctx.rc_wire(
                                            prev_cell,
                                            &format!(
                                                "JBLS{d}{k}_PREADD9_CORE_PREADD9_{prev_hl}{prev_idx}"
                                            ),
                                        );
                                        bel.claim_pip_to(
                                            wire_prev,
                                            format!("BLSO{k}_PREADD9_{hl}{i}"),
                                        );
                                        let wire_prev = bel.ctx.rc_wire(
                                            prev_cell,
                                            &format!("JAR{k}_MULT9_CORE_MULT9_{prev_hl}{prev_idx}"),
                                        );
                                        bel.claim_pip_from(
                                            format!("AS{d}{k}_MULT9_{hl}{i}"),
                                            wire_prev,
                                        );
                                    }
                                }
                            }
                            for k in 0..10 {
                                let idx = i * 20 + j * 10 + k;
                                bel.input_xlat_rename_rc(
                                    bcls::DSP::C[idx],
                                    format!("C{idx}_PREADD9"),
                                    format!("JC{k}_PREADD9_CORE_PREADD9_{hl}{ii}"),
                                );
                            }
                            for k in 0..20 {
                                bel.add_wire_rc(
                                    format!("P18_{k}_MULT9_{hl}{i}"),
                                    format!("JP18{k}_MULT9_CORE_MULT9_{hl}{ii}"),
                                )
                                .add_wire_rc(
                                    format!("P{hl}18_{k}_MULT18_{i}"),
                                    format!("JP{hl}18{k}_MULT18_CORE_MULT18_{ii}"),
                                )
                                .claim_pip(
                                    format!("P{hl}18_{k}_MULT18_{i}"),
                                    format!("P18_{k}_MULT9_{hl}{i}"),
                                );
                            }
                            bel.input_xlat_rc(
                                bcls::DSP::SIGNEDA[i * 2 + j],
                                format!("JASIGNED_MULT9_CORE_MULT9_{hl}{ii}"),
                            )
                            .input_xlat_rc(
                                bcls::DSP::SIGNEDB[i * 2 + j],
                                format!("JBSIGNED_PREADD9_CORE_PREADD9_{hl}{ii}"),
                            )
                            .input_xlat_rename_rc(
                                bcls::DSP::MULT_CLK[i],
                                format!("MULT_CLK{i}_PREADD_{hl}"),
                                format!("JCLK_PREADD9_CORE_PREADD9_{hl}{ii}"),
                            )
                            .input_xlat_rename_rc(
                                bcls::DSP::MULT_CLK[i],
                                format!("MULT_CLK{i}_MULT9_{hl}"),
                                format!("JCLK_MULT9_CORE_MULT9_{hl}{ii}"),
                            )
                            .input_xlat_rename_rc(
                                bcls::DSP::RSTA[i],
                                format!("RSTA{i}_MULT9_{hl}"),
                                format!("JRSTA_MULT9_CORE_MULT9_{hl}{ii}"),
                            )
                            .input_xlat_rename_rc(
                                bcls::DSP::CEA[i],
                                format!("CEA{i}_MULT9_{hl}"),
                                format!("JCEA_MULT9_CORE_MULT9_{hl}{ii}"),
                            )
                            .input_xlat_rename_rc(
                                bcls::DSP::RSTB[i],
                                format!("RSTB{i}_PREADD_{hl}"),
                                format!("JRSTB_PREADD9_CORE_PREADD9_{hl}{ii}"),
                            )
                            .input_xlat_rename_rc(
                                bcls::DSP::CEB[i],
                                format!("CEB{i}_PREADD_{hl}"),
                                format!("JCEB_PREADD9_CORE_PREADD9_{hl}{ii}"),
                            )
                            .input_xlat_rename_rc(
                                bcls::DSP::RSTCL[i],
                                format!("RSTCL{i}_PREADD_{hl}"),
                                format!("JRSTCL_PREADD9_CORE_PREADD9_{hl}{ii}"),
                            )
                            .input_xlat_rename_rc(
                                bcls::DSP::CECL[i],
                                format!("CECL{i}_PREADD_{hl}"),
                                format!("JCECL_PREADD9_CORE_PREADD9_{hl}{ii}"),
                            )
                            .input_xlat_rename_rc(
                                bcls::DSP::RSTP[i],
                                format!("RSTP{i}_MULT9_{hl}"),
                                format!("JRSTP_MULT9_CORE_MULT9_{hl}{ii}"),
                            )
                            .input_xlat_rename_rc(
                                bcls::DSP::CEP[i],
                                format!("CEP{i}_MULT9_{hl}"),
                                format!("JCEP_MULT9_CORE_MULT9_{hl}{ii}"),
                            );
                            for pname in [
                                "PRCASOUT", "PRCASIN", "BLSOSGND", "BRSOSGND", "BLSS1", "BLSS2",
                                "BRSS1", "BRSS2", "BRSIGNED",
                            ] {
                                bel.add_wire_rc(
                                    format!("{pname}_PREADD9_{hl}{i}"),
                                    format!("J{pname}_PREADD9_CORE_PREADD9_{hl}{ii}"),
                                );
                            }
                            for pname in [
                                "BRSIGNED",
                                "ARSIGNED",
                                "ASSIGNED1",
                                "ASSIGNED2",
                                "AOSIGNED",
                                "BOSIGNED",
                            ] {
                                bel.add_wire_rc(
                                    format!("{pname}_MULT9_{hl}{i}"),
                                    format!("J{pname}_MULT9_CORE_MULT9_{hl}{ii}"),
                                );
                            }
                            bel.claim_pip(
                                format!("BRSIGNED_MULT9_{hl}{i}"),
                                format!("BRSIGNED_PREADD9_{hl}{i}"),
                            );
                            if hl == 'H' {
                                for ab in ['A', 'B'] {
                                    bel.add_wire_rc(
                                        format!("{ab}R{hl}SIGN_MULT18_{i}"),
                                        format!("J{ab}R{hl}SIGN_MULT18_CORE_MULT18_{ii}"),
                                    )
                                    .claim_pip(
                                        format!("{ab}R{hl}SIGN_MULT18_{i}"),
                                        format!("{ab}OSIGNED_MULT9_{hl}{i}"),
                                    );
                                }
                            }
                            for (d, prev) in [(1, prev9), (2, prev9_2)] {
                                if let Some((prev_cell, prev_idx, prev_hl)) = prev {
                                    let wire_prev = bel.ctx.rc_wire(
                                        prev_cell,
                                        &format!(
                                            "JBRSOSGND_PREADD9_CORE_PREADD9_{prev_hl}{prev_idx}"
                                        ),
                                    );
                                    bel.claim_pip_from(
                                        format!("BRSS{d}_PREADD9_{hl}{i}"),
                                        wire_prev,
                                    );
                                    let wire_prev = bel.ctx.rc_wire(
                                        prev_cell,
                                        &format!(
                                            "JBLSS{d}_PREADD9_CORE_PREADD9_{prev_hl}{prev_idx}"
                                        ),
                                    );
                                    bel.claim_pip_to(
                                        wire_prev,
                                        format!("BLSOSGND_PREADD9_{hl}{i}"),
                                    );
                                    let wire_prev = bel.ctx.rc_wire(
                                        prev_cell,
                                        &format!("JARSIGNED_MULT9_CORE_MULT9_{prev_hl}{prev_idx}"),
                                    );
                                    bel.claim_pip_from(
                                        format!("ASSIGNED{d}_MULT9_{hl}{i}"),
                                        wire_prev,
                                    );
                                }
                            }
                            if let Some((prev_cell, prev_idx, prev_hl)) = prev9 {
                                let wire_prev = bel.ctx.rc_wire(
                                    prev_cell,
                                    &format!("JPRCASOUT_PREADD9_CORE_PREADD9_{prev_hl}{prev_idx}"),
                                );
                                bel.claim_pip_from(format!("PRCASIN_PREADD9_{hl}{i}"), wire_prev);
                            }

                            prev9_2 = prev9;
                            prev9 = Some((cur_cell, ii, hl));
                        }

                        bel.input_xlat_rc(
                            bcls::DSP::MULT_ROUNDEN[i],
                            format!("JROUNDEN_MULT18_CORE_MULT18_{ii}"),
                        );
                        for j in 0..4 {
                            bel.input_xlat_rc(
                                [bcls::DSP::MULT0_SFTCTRL, bcls::DSP::MULT1_SFTCTRL][i][j],
                                format!("JSFTCTRL{j}_MULT18_CORE_MULT18_{ii}"),
                            );
                        }
                        for j in 0..38 {
                            bel.add_wire_rc(
                                format!("P36_{j}_MULT18_{i}"),
                                format!("JP36{j}_MULT18_CORE_MULT18_{ii}"),
                            );
                        }
                        bel.add_wire_rc(
                            format!("SIGNED18_MULT18_{i}"),
                            format!("JSIGNED18_MULT18_CORE_MULT18_{ii}"),
                        );
                        for j in 0..18 {
                            bel.claim_pip(
                                format!("P36_{j}_MULT18_{i}"),
                                format!("PL18_{j}_MULT18_{i}"),
                            )
                            .claim_pip(
                                format!("P36_{jj}_MULT18_{i}", jj = j + 18),
                                format!("PH18_{j}_MULT18_{i}"),
                            );
                        }
                    }
                    bel.cell = cell.delta(3, 0);
                    bel.input_xlat_rc(
                        bcls::DSP::MULT18X36_ROUNDEN,
                        format!("JROUNDEN_MULT18X36_CORE_MULT18X36_{idx}"),
                    );
                    for j in 0..4 {
                        bel.input_xlat_rc(
                            bcls::DSP::MULT18X36_SFTCTRL[j],
                            format!("JSFTCTRL{j}_MULT18X36_CORE_MULT18X36_{idx}"),
                        );
                    }
                    for pin in ["SGNED18L", "SGNED18H"] {
                        bel.add_wire_rc(
                            format!("{pin}_MULT18X36"),
                            format!("J{pin}_MULT18X36_CORE_MULT18X36_{idx}"),
                        );
                    }
                    bel.claim_pip("SGNED18L_MULT18X36", "SIGNED18_MULT18_0")
                        .claim_pip("SGNED18H_MULT18X36", "SIGNED18_MULT18_1");
                    for i in 0..38 {
                        bel.add_wire_rc(
                            format!("PL36_{i}_MULT18X36"),
                            format!("JPL36{i}_MULT18X36_CORE_MULT18X36_{idx}"),
                        )
                        .add_wire_rc(
                            format!("PH36_{i}_MULT18X36"),
                            format!("JPH36{i}_MULT18X36_CORE_MULT18X36_{idx}"),
                        )
                        .claim_pip(format!("PL36_{i}_MULT18X36"), format!("P36_{i}_MULT18_0"))
                        .claim_pip(format!("PH36_{i}_MULT18X36"), format!("P36_{i}_MULT18_1"));
                    }
                    for i in 0..73 {
                        bel.add_wire_rc(
                            format!("P72_{i}_MULT18X36"),
                            format!("JP72{i}_MULT18X36_CORE_MULT18X36_{idx}"),
                        );
                    }
                    for i in 0..36 {
                        bel.claim_pip(format!("P72_{i}_MULT18X36"), format!("PL36_{i}_MULT18X36"))
                            .claim_pip(
                                format!("P72_{ii}_MULT18X36", ii = i + 36),
                                format!("PH36_{i}_MULT18X36"),
                            );
                    }
                    bel.cell = tcrd.delta(7, 0);
                    let hl = ['L', 'H'][idx];
                    for i in 0..73 {
                        bel.add_wire_rc(
                            format!("P72_{i}_MULT36"),
                            format!("JP{hl}72{i}_MULT36_CORE_MULT36"),
                        )
                        .claim_pip(format!("P72_{i}_MULT36"), format!("P72_{i}_MULT18X36"));
                    }
                    for i in 0..72 {
                        bel.add_wire_rc(
                            format!("PM72_{i}_MULT36"),
                            format!("JPM{hl}72{i}_MULT36_CORE_MULT36"),
                        )
                        .claim_pip(format!("PM72_{i}_MULT36"), format!("P72_{i}_MULT36"));
                    }
                    bel.cell = cell.delta(3, 0);
                    for i in 0..72 {
                        bel.add_wire_rc(
                            format!("PP{i}_ACC"),
                            format!("JPP{i}_ACC54_CORE_ACC54_{idx}"),
                        );
                    }
                    for i in 0..2 {
                        for j in 0..2 {
                            bel.input_xlat_rename_rc(
                                bcls::DSP::MULT_CLK[i],
                                format!("MULT_CLK{i}_REG18_{j}"),
                                format!("JCLK_REG18_CORE_REG18_{hl}{i}_{j}"),
                            )
                            .input_xlat_rename_rc(
                                bcls::DSP::RSTP[i],
                                format!("RSTP{i}_REG18_{j}"),
                                format!("JRSTP_REG18_CORE_REG18_{hl}{i}_{j}"),
                            )
                            .input_xlat_rename_rc(
                                bcls::DSP::CEP[i],
                                format!("CEP{i}_REG18_{j}"),
                                format!("JCEP_REG18_CORE_REG18_{hl}{i}_{j}"),
                            );
                            for k in 0..18 {
                                bel.add_wire_rc(
                                    format!("PM{k}_REG18_{i}_{j}"),
                                    format!("JPM{k}_REG18_CORE_REG18_{hl}{i}_{j}"),
                                )
                                .add_wire_rc(
                                    format!("PP{k}_REG18_{i}_{j}"),
                                    format!("JPP{k}_REG18_CORE_REG18_{hl}{i}_{j}"),
                                )
                                .claim_pip(
                                    format!("PM{k}_REG18_{i}_{j}"),
                                    format!("PM72_{ii}_MULT36", ii = i * 36 + j * 18 + k),
                                )
                                .claim_pip(
                                    format!("PP{k}_REG18_{i}_{j}"),
                                    format!("PM{k}_REG18_{i}_{j}"),
                                )
                                .claim_pip(
                                    format!("PP{ii}_ACC", ii = i * 36 + j * 18 + k),
                                    format!("PP{k}_REG18_{i}_{j}"),
                                );
                            }
                        }
                    }
                    bel.input_xlat_rc(bcls::DSP::ACC_CLK, format!("JCLK_ACC54_CORE_ACC54_{idx}"))
                        .input_xlat_rc(bcls::DSP::RSTC, format!("JRSTC_ACC54_CORE_ACC54_{idx}"))
                        .input_xlat_rc(bcls::DSP::CEC, format!("JCEC_ACC54_CORE_ACC54_{idx}"))
                        .input_xlat_rc(
                            bcls::DSP::RSTCTRL,
                            format!("JRSTCTRL_ACC54_CORE_ACC54_{idx}"),
                        )
                        .input_xlat_rc(bcls::DSP::CECTRL, format!("JCECTRL_ACC54_CORE_ACC54_{idx}"))
                        .input_xlat_rc(bcls::DSP::RSTCIN, format!("JRSTCIN_ACC54_CORE_ACC54_{idx}"))
                        .input_xlat_rc(bcls::DSP::CECIN, format!("JCECIN_ACC54_CORE_ACC54_{idx}"))
                        .input_xlat_rc(bcls::DSP::RSTO, format!("JRSTO_ACC54_CORE_ACC54_{idx}"))
                        .input_xlat_rc(bcls::DSP::CEO, format!("JCEO_ACC54_CORE_ACC54_{idx}"))
                        .input_xlat_rc(bcls::DSP::CIN, format!("JCIN_ACC54_CORE_ACC54_{idx}"))
                        .input_xlat_rc(bcls::DSP::LOAD, format!("JLOAD_ACC54_CORE_ACC54_{idx}"))
                        .input_xlat_rc(
                            bcls::DSP::ACC_SIGNEDI,
                            format!("JSIGNEDI_ACC54_CORE_ACC54_{idx}"),
                        )
                        .input_xlat_rc(
                            bcls::DSP::ACC_ROUNDEN,
                            format!("JROUNDEN_ACC54_CORE_ACC54_{idx}"),
                        );
                    for i in 0..4 {
                        bel.input_xlat_rc(
                            bcls::DSP::ACC_SFTCTRL[i],
                            format!("JSFTCTRL{i}_ACC54_CORE_ACC54_{idx}"),
                        );
                    }
                    for i in 0..2 {
                        bel.input_xlat_rc(
                            bcls::DSP::ADDSUB[i],
                            format!("JADDSUB{i}_ACC54_CORE_ACC54_{idx}"),
                        )
                        .input_xlat_rc(
                            bcls::DSP::M9ADDSUB[i],
                            format!("JM9ADDSUB{i}_ACC54_CORE_ACC54_{idx}"),
                        )
                        .add_wire_rc(
                            format!("CASIN{i}"),
                            format!("JCASIN{i}_ACC54_CORE_ACC54_{idx}"),
                        )
                        .add_wire_rc(
                            format!("CASCOUT{i}"),
                            format!("JCASCOUT{i}_ACC54_CORE_ACC54_{idx}"),
                        );
                    }
                    for i in 0..54 {
                        bel.input_xlat_rc(
                            bcls::DSP::C[i],
                            format!("JCINPUT{i}_ACC54_CORE_ACC54_{idx}"),
                        )
                        .add_wire_rc(
                            format!("DSPIN{i}"),
                            format!("JDSPIN{i}_ACC54_CORE_ACC54_{idx}"),
                        )
                        .add_wire_rc(
                            format!("DSPOUT{i}"),
                            format!("JDSPOUT{i}_ACC54_CORE_ACC54_{idx}"),
                        );
                    }
                    for i in 0..36 {
                        bel.output_xlat_rc(
                            bcls::DSP::SUM0[i],
                            format!("JSUM0{i}_ACC54_CORE_ACC54_{idx}"),
                        )
                        .output_xlat_rc(
                            bcls::DSP::SUM1[i],
                            format!("JSUM1{i}_ACC54_CORE_ACC54_{idx}"),
                        )
                        .claim_pip(format!("SUM0{i}"), format!("PP{i}_ACC"))
                        .claim_pip(format!("SUM1{i}"), format!("PP{ii}_ACC", ii = i + 36));
                    }
                    if let Some(prev) = prev_acc {
                        let prev_idx = idx ^ 1;
                        for i in 0..54 {
                            let wire_prev = bel
                                .ctx
                                .rc_wire(prev, &format!("JDSPOUT{i}_ACC54_CORE_ACC54_{prev_idx}"));
                            bel.claim_pip_from(format!("DSPIN{i}"), wire_prev);
                        }
                        for i in 0..2 {
                            let wire_prev = bel
                                .ctx
                                .rc_wire(prev, &format!("JCASCOUT{i}_ACC54_CORE_ACC54_{prev_idx}"));
                            bel.claim_pip_from(format!("CASIN{i}"), wire_prev);
                        }
                    }
                }
            }
        }
    }
}
