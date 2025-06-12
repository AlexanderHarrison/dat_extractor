const MAP: &[usize] = &[
    046, // Attack11                   
    047, // Attack12                   
    048, // Attack13                      => 
    049, // Attack100Start              
    050, // Attack100Loop               
    051, // Attack100End                
    052, // AttackDash                  
    053, // AttackS3Hi                  
    054, // AttackS3HiS                 
    055, // AttackS3S                   
    056, // AttackS3LwS                 
    057, // AttackS3Lw                  
    058, // AttackHi3                   
    059, // AttackLw3                   
    060, // AttackS4Hi                    => AttackS4
    061, // AttackS4HiS                   => AttackS4
    062, // AttackS4S                     => AttackS4
    063, // AttackS4LwS                   => AttackS4
    064, // AttackS4Lw                    => AttackS4
    066, // AttackHi4                    
    067, // AttackLw4                    
    068, // AttackAirN                   
    069, // AttackAirF                   
    070, // AttackAirB                   
    071, // AttackAirHi                  
    072, // AttackAirLw                  
];

fn main() {
    let file = std::fs::File::open("/home/alex/melee/melee_vanilla.iso").unwrap();
    let mut files = dat_tools::isoparser::ISODatFiles::new(file).unwrap();
    
    println!("// half open range 44..70 (Attack11..LandingAirN) ");
    println!("pub const ATTACK_RANGE_START: usize = 44;");
    println!("pub const ATTACK_RANGE_END: usize = 70;");
    println!();
    println!("// indexed by character, then by state_id - ATTACK_RANGE_START");
    println!("pub const ATTACK_HITBOXES: &[&[std::ops::Range<u32>]] = &[");
    
    for ch in slp_parser::Character::AS_LIST.iter() {
        let data = dat_tools::get_fighter_data(&mut files, ch.neutral()).unwrap();
        
        println!("    // {}", ch);
        println!("    &[");
        
        for ac_i in 44..70 {
            let action = MAP[ac_i - 44];
             
            let mut h_start = u32::MAX;
            let mut h_end = 0u32;
            if let Some(subactions) = data.action_table[action].subactions.as_ref() {
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
            let name = data.action_table[action].name.as_ref()
                .and_then(|n| dat_tools::dat::demangle_anim_name(n))
                .unwrap_or("");
            
            println!("        {}..{}, // {}", h_start, h_end, name);
        }
        
        println!("    ],");
    }
    println!("];");
}
