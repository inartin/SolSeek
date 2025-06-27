use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::bs58;
use std::time::{Instant, Duration};
use std::sync::{Arc, atomic::{AtomicU64, AtomicBool, Ordering}};
use std::thread;
use std::io::{self, Write};
use rayon::prelude::*;
use bip39::{Mnemonic, Language};

// Helper function to format large numbers
fn format_number(num: f64) -> String {
    if num >= 1_000_000_000.0 {
        format!("{:.2}B", num / 1_000_000_000.0)
    } else if num >= 1_000_000.0 {
        format!("{:.2}M", num / 1_000_000.0)
    } else if num >= 1_000.0 {
        format!("{:.2}K", num / 1_000.0)
    } else {
        format!("{:.0}", num)
    }
}

// Helper function to format time duration in readable format
fn format_duration(seconds: f64) -> String {
    let total_seconds = seconds as u64;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let secs = total_seconds % 60;
    let millis = ((seconds - total_seconds as f64) * 1000.0) as u64;

    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else if secs > 0 {
        format!("{}.{:03}s", secs, millis)
    } else {
        format!("{:.3}s", seconds)
    }
}

const BATCH_SIZE: usize = 100_000; // Optimized batch size for Rust

// Default Possible patterns to search for
const POSSIBLE_PATTERNS: &[&str] = &["Seek"];

#[derive(Debug, Clone)]
enum MatchPosition {
    StartOnly,
    EndOnly,
    StartOrEnd,
    Anywhere,
    StartAndEnd, // New option for both start and end patterns
}

#[derive(Debug, Clone)]
struct PatternData {
    pattern: String,
    length: usize,
    lower_pattern: String,
    compare_pattern: String,
}

#[derive(Debug, Clone)]
struct SearchConfig {
    start_pattern: Option<PatternData>,
    end_pattern: Option<PatternData>,
    match_position: MatchPosition,
    case_sensitive: bool,
}

impl PatternData {
    fn new(pattern: &str, case_sensitive: bool) -> Self {
        Self {
            pattern: pattern.to_string(),
            length: pattern.len(),
            lower_pattern: pattern.to_lowercase(),
            compare_pattern: if case_sensitive { pattern.to_string() } else { pattern.to_lowercase() },
        }
    }
}

fn validate_prefix(prefix: &str) -> bool {
    let base58_alphabet = "123456789ABCDEFGHIJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    prefix.chars().all(|c| base58_alphabet.contains(c))
}

fn check_address_with_config(address: &str, config: &SearchConfig) -> Option<(String, String)> {
    let compare_address = if config.case_sensitive { address } else { &address.to_lowercase() };

    match &config.match_position {
        MatchPosition::StartAndEnd => {
            // Both start and end patterns must match
            if let (Some(start_pattern), Some(end_pattern)) = (&config.start_pattern, &config.end_pattern) {
                let start_matches = compare_address.starts_with(&start_pattern.compare_pattern);
                let end_matches = compare_address.ends_with(&end_pattern.compare_pattern);

                if start_matches && end_matches {
                    let start_match = &address[..start_pattern.length];
                    let end_match = &address[address.len() - end_pattern.length..];
                    Some((format!("start '{}' and end '{}'", start_match, end_match),
                          format!("{} ... {}", start_match, end_match)))
                } else {
                    None
                }
            } else {
                None
            }
        }
        _ => {
            // For backward compatibility, use the first available pattern
            let pattern = config.start_pattern.as_ref().or(config.end_pattern.as_ref())?;
            check_address_single_pattern(address, pattern, &config.match_position, config.case_sensitive)
        }
    }
}

fn check_address_single_pattern(address: &str, pattern: &PatternData, match_position: &MatchPosition, case_sensitive: bool) -> Option<(String, String)> {
    let compare_address = if case_sensitive { address } else { &address.to_lowercase() };
    let compare_pattern = &pattern.compare_pattern;

    match match_position {
        MatchPosition::StartOnly => {
            if compare_address.starts_with(compare_pattern) {
                Some(("start".to_string(), address[..pattern.length].to_string()))
            } else {
                None
            }
        }
        MatchPosition::EndOnly => {
            if compare_address.ends_with(compare_pattern) {
                Some(("end".to_string(), address[address.len() - pattern.length..].to_string()))
            } else {
                None
            }
        }
        MatchPosition::StartOrEnd => {
            if compare_address.starts_with(compare_pattern) {
                Some(("start".to_string(), address[..pattern.length].to_string()))
            } else if compare_address.ends_with(compare_pattern) {
                Some(("end".to_string(), address[address.len() - pattern.length..].to_string()))
            } else {
                None
            }
        }
        MatchPosition::Anywhere => {
            if let Some(index) = compare_address.find(compare_pattern) {
                Some((format!("index {}", index), address[index..index + pattern.length].to_string()))
            } else {
                None
            }
        }
        MatchPosition::StartAndEnd => {
            // This should not be called for StartAndEnd, but handle it gracefully
            None
        }
    }
}

fn main() {
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();

    // Initialize search configuration
    let mut start_pattern_str: Option<String> = None;
    let mut end_pattern_str: Option<String> = None;
    let mut match_position = MatchPosition::StartOrEnd; // Default
    let mut case_sensitive = true; // Default to true (same as CASE_SENSITIVE constant)
    let mut pattern_args = Vec::new();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--start" if i + 1 < args.len() => {
                start_pattern_str = Some(args[i + 1].clone());
                i += 2;
            }
            "--end" if i + 1 < args.len() => {
                end_pattern_str = Some(args[i + 1].clone());
                i += 2;
            }
            "--position" if i + 1 < args.len() => {
                match args[i + 1].to_lowercase().as_str() {
                    "start" => match_position = MatchPosition::StartOnly,
                    "end" => match_position = MatchPosition::EndOnly,
                    "startorend" | "start-or-end" => match_position = MatchPosition::StartOrEnd,
                    "anywhere" => match_position = MatchPosition::Anywhere,
                    _ => {
                        eprintln!("Invalid position: {}. Use: start, end, startorend, anywhere", args[i + 1]);
                        std::process::exit(1);
                    }
                }
                i += 2;
            }
            "--case-sensitive" if i + 1 < args.len() => {
                match args[i + 1].to_lowercase().as_str() {
                    "true" | "1" | "yes" => case_sensitive = true,
                    "false" | "0" | "no" => case_sensitive = false,
                    _ => {
                        eprintln!("Invalid case-sensitive value: {}. Use: true, false, 1, 0, yes, no", args[i + 1]);
                        std::process::exit(1);
                    }
                }
                i += 2;
            }
            _ => {
                pattern_args.push(args[i].clone());
                i += 1;
            }
        }
    }

    // Determine search configuration
    let search_configs: Vec<SearchConfig> = if start_pattern_str.is_some() || end_pattern_str.is_some() {
        // Using --start and/or --end parameters
        let start_pattern = start_pattern_str.as_ref().map(|s| PatternData::new(s, case_sensitive));
        let end_pattern = end_pattern_str.as_ref().map(|s| PatternData::new(s, case_sensitive));

        // Validate patterns
        if let Some(ref pattern) = start_pattern {
            if !validate_prefix(&pattern.pattern) {
                eprintln!("Error: Start pattern \"{}\" contains invalid Base58 characters", pattern.pattern);
                eprintln!("Valid characters are: 123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz");
                std::process::exit(1);
            }
        }
        if let Some(ref pattern) = end_pattern {
            if !validate_prefix(&pattern.pattern) {
                eprintln!("Error: End pattern \"{}\" contains invalid Base58 characters", pattern.pattern);
                eprintln!("Valid characters are: 123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz");
                std::process::exit(1);
            }
        }

        let final_match_position = if start_pattern.is_some() && end_pattern.is_some() {
            MatchPosition::StartAndEnd
        } else if start_pattern.is_some() {
            MatchPosition::StartOnly
        } else {
            MatchPosition::EndOnly
        };

        vec![SearchConfig {
            start_pattern,
            end_pattern,
            match_position: final_match_position,
            case_sensitive,
        }]
    } else {
        // Use patterns from command line or default (backward compatibility)
        let has_custom_patterns = !pattern_args.is_empty();
        let possible_patterns: Vec<String> = if has_custom_patterns {
            pattern_args
        } else {
            POSSIBLE_PATTERNS.iter().map(|s| s.to_string()).collect()
        };

        // Validate all patterns
        for pattern in &possible_patterns {
            if !validate_prefix(pattern) {
                eprintln!("Error: Pattern \"{}\" contains invalid Base58 characters", pattern);
                eprintln!("Valid characters are: 123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz");
                std::process::exit(1);
            }
        }

        // Create search configs for backward compatibility
        possible_patterns.iter().map(|pattern| {
            let pattern_data = PatternData::new(pattern, case_sensitive);
            SearchConfig {
                start_pattern: Some(pattern_data.clone()),
                end_pattern: None,
                match_position: match_position.clone(),
                case_sensitive,
            }
        }).collect()
    };

    let start_time = Instant::now();
    let attempts = Arc::new(AtomicU64::new(0));
    let found = Arc::new(AtomicBool::new(false));
    let found_keypair = Arc::new(parking_lot::Mutex::new(None::<(String, String, String, String, String, String)>));

    println!("Solana Vanity Address Generator");

    // Display search configuration
    if start_pattern_str.is_some() || end_pattern_str.is_some() {
        if let Some(ref start) = start_pattern_str {
            println!("Start pattern: {}", start);
        }
        if let Some(ref end) = end_pattern_str {
            println!("End pattern: {}", end);
        }
        if start_pattern_str.is_some() && end_pattern_str.is_some() {
            println!("Mode: Both start and end patterns must match");
        }
    } else {
        let patterns: Vec<String> = search_configs.iter()
            .filter_map(|config| config.start_pattern.as_ref().map(|p| p.pattern.clone()))
            .collect();

        if patterns.len() == 1 && patterns[0] == POSSIBLE_PATTERNS[0] {
            println!("Using default patterns: {}", patterns.join(", "));
            println!("Usage: ./target/release/solana_vanity_generator [--start PATTERN] [--end PATTERN] [--case-sensitive true|false] [--position start|end|startorend|anywhere] [PATTERN1 PATTERN2 ...]");
        } else {
            println!("Using command line patterns: {}", patterns.join(", "));
        }
        println!("Match position: {:?}", match_position);
    }

    println!("Case sensitive: {}", case_sensitive);
    println!("Using {} CPU threads with batch size {}", num_cpus::get(), BATCH_SIZE);

    // Progress reporting thread
    let attempts_clone = Arc::clone(&attempts);
    let found_clone = Arc::clone(&found);
    let start_time_clone = start_time;
    thread::spawn(move || {
        let mut last_report_time = start_time_clone;
        let mut last_report_attempts = 0u64;

        while !found_clone.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_secs(1));
            let now = Instant::now();
            let elapsed = now.duration_since(start_time_clone).as_secs_f64();
            let total_attempts = attempts_clone.load(Ordering::Relaxed);

            let time_span = now.duration_since(last_report_time).as_secs_f64();
            let recent_attempts = total_attempts - last_report_attempts;
            let current_rate = if time_span > 0.0 { recent_attempts as f64 / time_span } else { 0.0 };
            let avg_rate = if elapsed > 0.0 { total_attempts as f64 / elapsed } else { 0.0 };

            print!("\rAttempts: {} | Current Rate: {} addr/sec | Avg Rate: {} addr/sec | Running: {}s",
                format_number(total_attempts as f64),
                format_number(current_rate),
                format_number(avg_rate),
                elapsed as u64
            );
            io::stdout().flush().unwrap();

            last_report_time = now;
            last_report_attempts = total_attempts;
        }
    });

    // Parallel keypair generation with batching
    (0..num_cpus::get()).into_par_iter().for_each(|_worker_id| {
        let mut local_attempts = 0u64;
        let report_threshold = 10_000u64;

        while !found.load(Ordering::Relaxed) {
            // Generate keypairs in batches for better performance
            for _ in 0..BATCH_SIZE {
                if found.load(Ordering::Relaxed) {
                    break;
                }

                let keypair = Keypair::new();
                let pubkey = keypair.pubkey().to_string();
                local_attempts += 1;

                // Check against all search configurations
                for config in &search_configs {
                    if let Some((position, actual_match)) = check_address_with_config(&pubkey, config) {
                        found.store(true, Ordering::Relaxed);
                        let private_key = bs58::encode(keypair.to_bytes()).into_string();

                        // Generate mnemonic phrase from keypair entropy
                        let keypair_bytes = keypair.to_bytes();
                        let mnemonic_phrase = match Mnemonic::from_entropy(&keypair_bytes[..32], Language::English) {
                            Ok(mnemonic) => mnemonic.phrase().to_string(),
                            Err(_) => "Unable to generate mnemonic".to_string(),
                        };

                        // Determine the matched pattern description
                        let pattern_desc = match (&config.start_pattern, &config.end_pattern) {
                            (Some(start), Some(end)) => format!("start '{}' + end '{}'", start.pattern, end.pattern),
                            (Some(start), None) => start.pattern.clone(),
                            (None, Some(end)) => end.pattern.clone(),
                            (None, None) => "unknown".to_string(),
                        };

                        *found_keypair.lock() = Some((
                            pattern_desc,
                            position,
                            actual_match,
                            pubkey,
                            private_key,
                            mnemonic_phrase
                        ));
                        attempts.fetch_add(local_attempts, Ordering::Relaxed);
                        return;
                    }
                }

                // Report progress periodically
                if local_attempts % report_threshold == 0 {
                    attempts.fetch_add(report_threshold, Ordering::Relaxed);
                    local_attempts = 0;
                }
            }

            // Report remaining attempts
            if local_attempts > 0 {
                attempts.fetch_add(local_attempts, Ordering::Relaxed);
                local_attempts = 0;
            }
        }
    });

    let result = found_keypair.lock().clone();
    if let Some((matched_pattern, position, actual_match, pubkey, private_key, mnemonic_phrase)) = result {
        let elapsed_time = start_time.elapsed().as_secs_f64();
        let total_attempts = attempts.load(Ordering::Relaxed);

        println!("\n\nFound matching address!");
        println!("Matched pattern: {}", matched_pattern);
        println!("Match position: {}", position);
        println!("Actual match: {}", actual_match);
        println!("Public Key: {}", pubkey);
        println!("Secret Key (Base58): {}", private_key);
        println!("Secret Key (Mnemonic): {}", mnemonic_phrase);
        println!("Total attempts: {}", total_attempts);
        println!("Time taken: {}", format_duration(elapsed_time));
        println!("Speed: {} addresses/second", format_number(total_attempts as f64 / elapsed_time));
    }
}
