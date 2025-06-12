fn main() {
    let file = std::fs::File::open("/home/alex/melee/melee_vanilla.iso").unwrap();
    let mut files = dat_tools::isoparser::ISODatFiles::new(file).unwrap();
    
    println!("// half open range 46..73 (Attack11..LandingAirN) ");
    println!("pub const ATTACK_RANGE_START: usize = 46");
    println!("pub const ATTACK_RANGE_END: usize = 73");
    println!();
    println!("// indexed by character, then by action state - ATTACK_RANGE_START");
    println!("pub const ATTACK_HITBOXES: &[&[std::ops::Range<u32>]] = &[");
    
    for ch in slp_parser::Character::AS_LIST.iter() {
        let data = dat_tools::get_fighter_data(&mut files, ch.neutral()).unwrap();
        
        let mut ranges = Vec::new();
        for ac_i in 46..=72 {
            let mut h_start = u32::MAX;
            let mut h_end = 0u32;
            if let Some(subactions) = data.action_table[ac_i].subactions.as_ref() {
                let mut f = 0;
                let mut i = 0;
                let mut loop_start = 0usize;
                let mut loop_i = 0usize;
                while i < subactions.len() {
                    let word = subactions[i];
                    let cmd = dat_tools::dat::subaction_cmd(word);
            
                    use dat_tools::dat::Subaction as S;
                    match dat_tools::dat::parse_next_subaction(&subactions[i..]) {
                        S::EndOfScript => break,
                        S::AsynchronousTimer { frame } => f = frame as usize,
                        S::SynchronousTimer { frame } => f += frame as usize,
        
                        S::SetLoop { loop_count } => {
                            loop_start = i + dat_tools::dat::subaction_size(cmd);
                            loop_i = loop_count as usize - 1;
                        }
                        S::ExecuteLoop if loop_i != 0 => {
                            loop_i -= 1;
                            i = loop_start;
        
                            // skip index increment
                            continue;
                        }
                        
                        S::CreateHitbox { .. } => h_start = h_start.min(f as u32),
                        S::ClearHitboxes => h_end = h_end.max(f as u32),
                        _ => (),
                    }
                    
                    i += dat_tools::dat::subaction_size(cmd);
                }
            }
                
            if h_end == 0 { h_end = u32::MAX; }
            if h_start == u32::MAX { h_start = 0; }
            ranges.push((h_start, h_end));
        }
        
        println!("    // {}", ch);
        println!("    &[");
        for r in ranges {
            println!("        {}..{},", r.0, r.1);
        }
        println!("    ],");
    }
    println!("];");
}
